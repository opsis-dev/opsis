use std::future::Future;
use std::sync::Arc;

use rmcp::model::{
    CallToolRequestParams, CallToolResponse, Implementation, ListToolsResult,
    PaginatedRequestParams, ServerCapabilities, ServerConfig, Tool,
};
use rmcp::service::{MaybeSendFuture, RequestContext};
use rmcp::{ErrorData, RoleServer, ServerHandler};

use crate::local_api::LocalApi;
use crate::tools;

pub const GUIDE: &str = include_str!("../guide.md");

#[derive(Debug, Clone)]
pub struct OpsisTools {
    api: LocalApi,
    tools: Arc<Vec<Tool>>,
}

impl OpsisTools {
    pub fn new(api: LocalApi) -> Self {
        Self {
            api,
            tools: Arc::new(tools::assistant_tools()),
        }
    }
}

impl ServerHandler for OpsisTools {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("opsis", env!("CARGO_PKG_VERSION")))
            .with_instructions(GUIDE)
    }

    fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListToolsResult, ErrorData>> + MaybeSendFuture + '_ {
        std::future::ready(Ok(ListToolsResult::with_all_items(self.tools.to_vec())))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, ErrorData> {
        let arguments = request.arguments.unwrap_or_default();
        Ok(tools::call(&self.api, &request.name, &arguments)
            .await
            .into())
    }
}
