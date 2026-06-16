pub mod server;
pub mod tools;

use crate::ipc::{client::IpcClient, server::read_session_id};
use anyhow::{Context, Result};
use rmcp::ServiceExt;
use server::DevTerminalMcpServer;
use std::process::Stdio;
use std::time::Duration;
use tracing::info;

/// Entry point for `holocron --mcp`.
/// Bridges MCP over stdio → IPC → a running terminal session. If no session
/// is running (or the one on disk is stale), a headless session is spawned
/// automatically so the bridge always has something to connect to.
pub async fn run_bridge(session: Option<String>) -> Result<()> {
    let session_id = match session {
        Some(id) => id,
        None => resolve_or_spawn_session().await?,
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

async fn resolve_or_spawn_session() -> Result<String> {
    if let Ok(id) = read_session_id() {
        if IpcClient::new(id.clone()).is_alive().await {
            return Ok(id);
        }
        info!("session {id} on disk is stale (socket unreachable), starting a fresh headless session");
    }

    spawn_headless_session().await
}

/// Spawns `holocron --headless` as a detached background process and waits
/// for it to come up before handing back its session ID.
async fn spawn_headless_session() -> Result<String> {
    crate::ipc::server::cleanup_session_file();

    let exe = std::env::current_exe().context("failed to resolve holocron executable path")?;
    let mut cmd = std::process::Command::new(exe);
    cmd.arg("--headless")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    detach(&mut cmd);

    cmd.spawn().context("failed to spawn headless holocron session")?;

    for _ in 0..50 {
        tokio::time::sleep(Duration::from_millis(100)).await;
        if let Ok(id) = read_session_id() {
            if IpcClient::new(id.clone()).is_alive().await {
                info!("headless session {id} is up");
                return Ok(id);
            }
        }
    }

    anyhow::bail!("timed out waiting for headless holocron session to start")
}

/// Detaches the child from this process's session/process group so it
/// keeps running independently of the (short-lived) MCP bridge process.
#[cfg(unix)]
fn detach(cmd: &mut std::process::Command) {
    use std::os::unix::process::CommandExt;
    cmd.process_group(0);
}

#[cfg(windows)]
fn detach(cmd: &mut std::process::Command) {
    use std::os::windows::process::CommandExt;
    const DETACHED_PROCESS: u32 = 0x0000_0008;
    const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
    cmd.creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP);
}

#[cfg(not(any(unix, windows)))]
fn detach(_cmd: &mut std::process::Command) {}
