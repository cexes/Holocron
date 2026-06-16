use crate::{
    config::Config,
    ipc::server::{cleanup_session_file, run_ipc_server, write_session_file},
    terminal::manager::TerminalManager,
};
use anyhow::Result;
use std::sync::{Arc, Mutex};
use tracing::info;
use uuid::Uuid;

/// Entry point for `holocron --headless`.
/// Runs the terminal manager + IPC server with no TUI attached, so the MCP
/// bridge has something to connect to even when no one opened the TUI.
pub async fn run(config: Config) -> Result<()> {
    let session_id = Uuid::new_v4().to_string();

    let (pty_tx, mut pty_rx) = tokio::sync::mpsc::unbounded_channel();
    let manager: Arc<Mutex<TerminalManager>> =
        Arc::new(Mutex::new(TerminalManager::new(config.shell_command(), pty_tx)));

    write_session_file(&session_id)?;
    info!("headless session {session_id} started");

    // Nothing renders in headless mode, so PTY output notifications must
    // still be drained or the unbounded channel grows forever.
    tokio::spawn(async move { while pty_rx.recv().await.is_some() {} });

    let result = tokio::select! {
        res = run_ipc_server(session_id.clone(), manager) => res,
        _ = shutdown_signal() => Ok(()),
    };

    cleanup_session_file();
    result
}

#[cfg(unix)]
async fn shutdown_signal() -> Result<()> {
    let mut term = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {}
        _ = term.recv() => {}
    }
    Ok(())
}

#[cfg(not(unix))]
async fn shutdown_signal() -> Result<()> {
    tokio::signal::ctrl_c().await?;
    Ok(())
}
