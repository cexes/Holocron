pub mod server;
pub mod tools;

use crate::ipc::server::read_session_id;
use anyhow::{Context, Result};
use rmcp::ServiceExt;
use server::DevTerminalMcpServer;
use tracing::info;

/// Entry point for `holocron --mcp`.
/// Bridges MCP over stdio → IPC → running TUI instance.
pub async fn run_bridge(session: Option<String>) -> Result<()> {
    let session_id = match session {
        Some(id) => id,
        None => read_session_id()
            .context("no holocron session found — start holocron first")?,
    };

    info!("MCP bridge connecting to session {session_id}");

    let mcp_server = DevTerminalMcpServer::new(session_id);

    // Use tokio stdio as transport (stdin/stdout)
    let transport = (tokio::io::stdin(), tokio::io::stdout());

    mcp_server
        .serve(transport)
        .await
        .context("MCP server error")?
        .waiting()
        .await?;

    Ok(())
}
