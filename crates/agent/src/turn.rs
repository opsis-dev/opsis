use std::path::PathBuf;

use futures::StreamExt;
use rig_core::DynModel;
use rig_core::completion::CompletionRequest;
use rig_core::message::{AssistantContent, Message, ToolCall, ToolResult, ToolResultContent};
use rig_core::operation::Completion;
use rig_core::streaming::StreamEvent;
use serde_json::Value;
use tokio_util::sync::CancellationToken;

use crate::mcp::ToolServers;
use crate::tools;

#[derive(Debug, Clone, PartialEq)]
pub enum Update {
    Text(String),
    ToolStarted {
        id: String,
        title: String,
        reads: bool,
        input: Value,
    },
    ToolEnded {
        id: String,
        ok: bool,
        output: Vec<ToolResultContent>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ending {
    Done,
    Cancelled,
    Refused,
}

pub trait Peer: Send + Sync {
    fn update(&self, update: Update);
    fn allow(&self, id: &str, title: &str, input: &Value) -> impl Future<Output = bool> + Send;
}

#[derive(Debug, thiserror::Error)]
pub enum TurnError {
    #[error("The model service answered with an error: {0}")]
    Model(String),
}

#[derive(Debug)]
pub struct Turn<'a> {
    pub folder: PathBuf,
    pub system: String,
    pub model: &'a DynModel<Completion>,
    pub servers: &'a ToolServers,
    pub cancel: CancellationToken,
}

fn request(
    system: &str,
    history: &[Message],
    tools: &[rig_core::completion::ToolDefinition],
) -> Option<CompletionRequest> {
    let (last, earlier) = history.split_last()?;
    Some(
        CompletionRequest::new(last.clone())
            .messages(earlier.iter().cloned())
            .preamble(system)
            .tools(tools.iter().cloned()),
    )
}

fn calls(message: &Message) -> Vec<ToolCall> {
    match message {
        Message::Assistant(turn) => turn
            .content
            .iter()
            .filter_map(|block| match block {
                AssistantContent::ToolCall(call) => Some(call.clone()),
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    }
}

impl Turn<'_> {
    fn needs_asking(&self, name: &str) -> bool {
        if tools::is_local(name) {
            tools::effect(name) == tools::Effect::Changes
        } else {
            !self.servers.reads_only(name)
        }
    }

    async fn run_tool(&self, call: &ToolCall) -> (bool, Vec<ToolResultContent>) {
        let name = call.function.name.as_ref();
        let arguments = call.function.arguments.clone();
        if tools::is_local(name) {
            match tools::run(&self.folder, name, &arguments) {
                Ok(content) => (true, content),
                Err(error) => (false, vec![ToolResultContent::text(error.to_string())]),
            }
        } else {
            match self.servers.call(name, arguments).await {
                Ok(text) => (true, vec![ToolResultContent::text(text)]),
                Err(text) => (false, vec![ToolResultContent::text(text)]),
            }
        }
    }

    async fn answer(&self, call: &ToolCall, peer: &impl Peer) -> Result<ToolResult, Ending> {
        let id = call.id.to_string();
        let name = call.function.name.to_string();
        let input = Value::Object(call.function.arguments.clone());
        let asks = self.needs_asking(&name);
        peer.update(Update::ToolStarted {
            id: id.clone(),
            title: name.clone(),
            reads: !asks,
            input: input.clone(),
        });
        if asks {
            let allowed = tokio::select! {
                allowed = peer.allow(&id, &name, &input) => allowed,
                () = self.cancel.cancelled() => return Err(Ending::Cancelled),
            };
            if !allowed {
                let refused = vec![ToolResultContent::text("The person didn't allow this.")];
                peer.update(Update::ToolEnded {
                    id,
                    ok: false,
                    output: refused,
                });
                return Err(Ending::Refused);
            }
        }
        let (ok, output) = tokio::select! {
            ran = self.run_tool(call) => ran,
            () = self.cancel.cancelled() => return Err(Ending::Cancelled),
        };
        peer.update(Update::ToolEnded {
            id,
            ok,
            output: output.clone(),
        });
        let mut result = call.result(output);
        result.is_error = !ok;
        Ok(result)
    }

    async fn reply(&self, history: &[Message], peer: &impl Peer) -> Result<Message, TurnError> {
        let mut definitions = tools::definitions();
        definitions.extend(self.servers.definitions());
        let request = request(&self.system, history, &definitions)
            .ok_or_else(|| TurnError::Model("nothing to answer".to_owned()))?;
        let mut stream = self
            .model
            .stream(request)
            .map_err(|e| TurnError::Model(e.to_string()))?;
        while let Some(item) = stream.next().await {
            let item = item.map_err(|e| TurnError::Model(e.to_string()))?;
            if let rig_core::streaming::Item::Event(StreamEvent::Text { text, .. }) = item {
                peer.update(Update::Text(text));
            }
        }
        let response = stream
            .finish()
            .await
            .map_err(|e| TurnError::Model(e.to_string()))?;
        if let Some(error) = response.error {
            return Err(TurnError::Model(error));
        }
        Ok(Message::Assistant(
            rig_core::message::AssistantMessage::new(response.choice),
        ))
    }

    pub async fn run(
        &self,
        history: &mut Vec<Message>,
        peer: &impl Peer,
        mut saved: impl FnMut(&Message),
    ) -> Result<Ending, TurnError> {
        loop {
            if self.cancel.is_cancelled() {
                return Ok(Ending::Cancelled);
            }
            let reply = tokio::select! {
                reply = self.reply(history, peer) => reply?,
                () = self.cancel.cancelled() => return Ok(Ending::Cancelled),
            };
            let calls = calls(&reply);
            saved(&reply);
            history.push(reply);
            if calls.is_empty() {
                return Ok(Ending::Done);
            }
            let mut results = Vec::new();
            let mut ended = None;
            for call in &calls {
                if ended.is_some() {
                    let mut skipped = call.result(vec![ToolResultContent::text(
                        "Not run: the turn ended first.",
                    )]);
                    skipped.is_error = true;
                    results.push(skipped);
                    continue;
                }
                match self.answer(call, peer).await {
                    Ok(result) => results.push(result),
                    Err(ending) => {
                        ended = Some(ending);
                        let mut stopped = call.result(vec![ToolResultContent::text("Not run.")]);
                        stopped.is_error = true;
                        results.push(stopped);
                    }
                }
            }
            let message = Message::tool_results(results);
            saved(&message);
            history.push(message);
            if let Some(ending) = ended {
                return Ok(ending);
            }
        }
    }
}
