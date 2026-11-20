use std::sync::Arc;

use rmcp::model::{
    CallToolRequestParams, CallToolResponse, Implementation, ListToolsResult,
    PaginatedRequestParams, ServerCapabilities, ServerConfig, Tool,
};
use rmcp::service::RequestContext;
use rmcp::{ErrorData, RoleServer, ServerHandler};

use crate::full::{call_route, fetch_route_tools};
use crate::local_api::LocalApi;
use crate::tools;

pub const GUIDE: &str = include_str!("../guide.md");

#[derive(Debug, Clone)]
pub struct OpsisTools {
    api: LocalApi,
    tools: Arc<Vec<Tool>>,
    full: bool,
}

impl OpsisTools {
    pub fn new(api: LocalApi, full: bool) -> Self {
        Self {
            api,
            tools: Arc::new(tools::assistant_tools()),
            full,
        }
    }

    fn is_assistant_tool(&self, name: &str) -> bool {
        self.tools.iter().any(|tool| tool.name == name)
    }
}

impl ServerHandler for OpsisTools {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("opsis", env!("CARGO_PKG_VERSION")))
            .with_instructions(GUIDE)
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        let mut listed = self.tools.to_vec();
        if self.full {
            match fetch_route_tools(&self.api).await {
                Ok(routes) => listed.extend(routes.into_iter().map(|route| route.tool)),
                Err(error) => log::warn!("listing every route: {error}"),
            }
        }
        Ok(ListToolsResult::with_all_items(listed))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, ErrorData> {
        let arguments = request.arguments.unwrap_or_default();
        if self.full && !self.is_assistant_tool(&request.name) {
            let routes = match fetch_route_tools(&self.api).await {
                Ok(routes) => routes,
                Err(error) => return Ok(tools::failure(&error).into()),
            };
            if let Some(route) = routes.iter().find(|route| route.tool.name == request.name) {
                return Ok(call_route(&self.api, route, &arguments).await.into());
            }
        }
        Ok(tools::call(&self.api, &request.name, &arguments)
            .await
            .into())
    }
}
