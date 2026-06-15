use crate::mcp::tools::TerminalTools;
use anyhow::Result;
use rmcp::{
    ErrorData as McpError,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::CallToolResult,
    schemars, tool, tool_handler, tool_router, ServerHandler,
};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Clone)]
pub struct DevTerminalMcpServer {
    pub tool_router: ToolRouter<Self>,
    tools: Arc<TerminalTools>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct ReadTerminalArgs {
    /// Terminal ID (UUID)
    id: String,
    /// Number of lines to return (default: 50)
    lines: Option<usize>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct SendCommandArgs {
    /// Terminal ID (UUID)
    id: String,
    /// Command text to send (Enter is appended automatically)
    command: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct TerminalIdArgs {
    /// Terminal ID (UUID)
    id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct CreateTerminalArgs {
    /// Human-readable label for the new terminal pane
    label: String,
    /// Optional shell command to run instead of the default shell
    command: Option<String>,
}

#[tool_router]
impl DevTerminalMcpServer {
    /// List all active terminal panes with their IDs, labels, and status.
    #[tool(description = "List all active terminal panes managed by holocron")]
    async fn list_terminals(&self) -> Result<CallToolResult, McpError> {
        self.tools
            .list_terminals()
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))
    }

    /// Read the last N lines of output from a terminal pane.
    #[tool(description = "Read recent output from a terminal pane")]
    async fn read_terminal(
        &self,
        Parameters(args): Parameters<ReadTerminalArgs>,
    ) -> Result<CallToolResult, McpError> {
        let id = args
            .id
            .parse()
            .map_err(|e| McpError::invalid_params(format!("invalid UUID: {e}"), None))?;
        self.tools
            .read_terminal(id, args.lines)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))
    }

    /// Send a command to a terminal pane (Enter is appended automatically).
    #[tool(description = "Send a command to a terminal pane")]
    async fn send_command(
        &self,
        Parameters(args): Parameters<SendCommandArgs>,
    ) -> Result<CallToolResult, McpError> {
        let id = args
            .id
            .parse()
            .map_err(|e| McpError::invalid_params(format!("invalid UUID: {e}"), None))?;
        self.tools
            .send_command(id, args.command)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))
    }

    /// Get detailed info about a terminal pane (label, dimensions, ID).
    #[tool(description = "Get detailed information about a specific terminal pane")]
    async fn get_terminal_info(
        &self,
        Parameters(args): Parameters<TerminalIdArgs>,
    ) -> Result<CallToolResult, McpError> {
        let id = args
            .id
            .parse()
            .map_err(|e| McpError::invalid_params(format!("invalid UUID: {e}"), None))?;
        self.tools
            .get_terminal_info(id)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))
    }

    /// Create a new terminal pane.
    #[tool(description = "Create a new terminal pane in the holocron session")]
    async fn create_terminal(
        &self,
        Parameters(args): Parameters<CreateTerminalArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.tools
            .create_terminal(args.label, args.command)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))
    }
}

impl DevTerminalMcpServer {
    pub fn new(session_id: String) -> Self {
        Self {
            tool_router: Self::tool_router(),
            tools: Arc::new(TerminalTools::new(session_id)),
        }
    }
}

#[tool_handler(instructions = "Provides access to terminal panes managed by holocron. Use list_terminals to discover panes, read_terminal to inspect output, and send_command to interact with them.")]
impl ServerHandler for DevTerminalMcpServer {}
