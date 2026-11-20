use std::path::Path;

use rig_core::completion::ToolDefinition;
use rig_core::message::ToolName;
use rmcp::model::{CallToolRequestParams, Tool};
use rmcp::service::RunningService;
use rmcp::transport::TokioChildProcess;
use rmcp::{RoleClient, ServiceExt};
use serde_json::{Map, Value};

const SEPARATOR: &str = "__";
const NAME_LIMIT: usize = 64;

pub struct ToolServer {
    name: String,
    client: RunningService<RoleClient, ()>,
    tools: Vec<Tool>,
    instructions: Option<String>,
}

impl std::fmt::Debug for ToolServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToolServer")
            .field("name", &self.name)
            .field("tools", &self.tools.len())
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StdioServer {
    pub name: String,
    pub command: std::path::PathBuf,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
}

#[derive(Debug, Default)]
pub struct ToolServers {
    servers: Vec<ToolServer>,
}

pub fn exposed_name(server: &str, tool: &str) -> String {
    let clean = |text: &str| -> String {
        text.chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '-' {
                    c
                } else {
                    '_'
                }
            })
            .collect()
    };
    let mut name = format!("{}{SEPARATOR}{}", clean(server), clean(tool));
    name.truncate(NAME_LIMIT);
    name
}

async fn start_one(server: &StdioServer, folder: &Path) -> Result<ToolServer, String> {
    let mut command = tokio::process::Command::new(&server.command);
    command.args(&server.args).current_dir(folder);
    for (name, value) in &server.env {
        command.env(name, value);
    }
    let transport = TokioChildProcess::new(command).map_err(|e| e.to_string())?;
    let client = ().serve(transport).await.map_err(|e| e.to_string())?;
    let tools = client.list_all_tools().await.map_err(|e| e.to_string())?;
    let instructions = client
        .peer_info()
        .and_then(|info| info.instructions.clone());
    Ok(ToolServer {
        name: server.name.clone(),
        client,
        tools,
        instructions,
    })
}

impl ToolServers {
    pub async fn start(servers: &[StdioServer], folder: &Path) -> Self {
        let mut started = Vec::new();
        for server in servers {
            match start_one(server, folder).await {
                Ok(server) => started.push(server),
                Err(error) => log::warn!("the tool server {} didn't start: {error}", server.name),
            }
        }
        Self { servers: started }
    }

    pub fn definitions(&self) -> Vec<ToolDefinition> {
        self.servers
            .iter()
            .flat_map(|server| {
                server.tools.iter().filter_map(|tool| {
                    Some(ToolDefinition {
                        name: ToolName::new(exposed_name(&server.name, &tool.name)).ok()?,
                        description: tool.description.as_deref().unwrap_or_default().to_owned(),
                        parameters: Value::Object(tool.input_schema.as_ref().clone()),
                        annotations: rig_core::completion::ToolAnnotations::default(),
                    })
                })
            })
            .collect()
    }

    pub fn instructions(&self) -> Vec<String> {
        self.servers
            .iter()
            .filter_map(|server| server.instructions.clone())
            .collect()
    }

    fn find(&self, exposed: &str) -> Option<(&ToolServer, &Tool)> {
        self.servers.iter().find_map(|server| {
            server
                .tools
                .iter()
                .find(|tool| exposed_name(&server.name, &tool.name) == exposed)
                .map(|tool| (server, tool))
        })
    }

    pub fn has(&self, exposed: &str) -> bool {
        self.find(exposed).is_some()
    }

    pub fn reads_only(&self, exposed: &str) -> bool {
        self.find(exposed)
            .and_then(|(_, tool)| tool.annotations.as_ref())
            .and_then(|annotations| annotations.read_only_hint)
            .unwrap_or(false)
    }

    pub async fn call(
        &self,
        exposed: &str,
        arguments: Map<String, Value>,
    ) -> Result<String, String> {
        let (server, tool) = self
            .find(exposed)
            .ok_or_else(|| format!("There is no tool {exposed}."))?;
        let result = server
            .client
            .call_tool(CallToolRequestParams::new(tool.name.clone()).with_arguments(arguments))
            .await
            .map_err(|e| e.to_string())?;
        let text: Vec<String> = result
            .content
            .iter()
            .filter_map(|block| block.as_text().map(|t| t.text.clone()))
            .collect();
        let text = text.join("\n");
        if result.is_error == Some(true) {
            Err(text)
        } else {
            Ok(text)
        }
    }

    pub async fn close(self) {
        for server in self.servers {
            if let Err(error) = server.client.cancel().await {
                log::debug!("closing the tool server {}: {error}", server.name);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    proptest! {
        #[test]
        fn an_exposed_name_is_what_every_model_service_accepts(server in "\\PC{0,40}", tool in "\\PC{0,40}") {
            let name = exposed_name(&server, &tool);
            prop_assert!(name.len() <= NAME_LIMIT);
            prop_assert!(name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'));
        }
    }
}
