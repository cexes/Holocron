use crate::{
    ipc::protocol::{IpcRequest, IpcResponse},
    terminal::manager::{SessionInfo, TerminalManager},
};
use anyhow::Result;
use interprocess::local_socket::{
    tokio::{prelude::*, Stream},
    GenericNamespaced, ListenerOptions, ToNsName,
};
use serde_json;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::{error, info};
use uuid::Uuid;

pub fn socket_name(session_id: &str) -> String {
    format!("holocron-{session_id}.sock")
}

pub async fn run_ipc_server(
    session_id: String,
    manager: Arc<Mutex<TerminalManager>>,
) -> Result<()> {
    let name = socket_name(&session_id)
        .to_ns_name::<GenericNamespaced>()?;

    let opts = ListenerOptions::new().name(name);
    let listener = opts.create_tokio()?;

    info!("IPC server listening on holocron-{session_id}.sock");

    loop {
        match listener.accept().await {
            Ok(conn) => {
                let mgr = Arc::clone(&manager);
                tokio::spawn(handle_connection(conn, mgr));
            }
            Err(e) => {
                error!("IPC accept error: {e}");
            }
        }
    }
}

async fn handle_connection(conn: Stream, manager: Arc<Mutex<TerminalManager>>) {
    let (reader, mut writer) = tokio::io::split(conn);
    let mut lines = BufReader::new(reader).lines();

    while let Ok(Some(line)) = lines.next_line().await {
        let response = match serde_json::from_str::<IpcRequest>(&line) {
            Ok(req) => handle_request(req, &manager),
            Err(e) => IpcResponse::error(format!("invalid request: {e}")),
        };

        let mut json = serde_json::to_string(&response).unwrap_or_default();
        json.push('\n');
        if writer.write_all(json.as_bytes()).await.is_err() {
            break;
        }
    }
}

fn handle_request(req: IpcRequest, manager: &Arc<Mutex<TerminalManager>>) -> IpcResponse {
    let Ok(mut mgr) = manager.lock() else {
        return IpcResponse::error("manager lock poisoned");
    };

    match req {
        IpcRequest::ListTerminals => {
            let list: Vec<SessionInfo> = mgr.list_info();
            IpcResponse::ok(list)
        }

        IpcRequest::ReadTerminal { id, lines } => {
            let n = lines.unwrap_or(50);
            if let Some(session) = mgr.get_mut(id) {
                let output: Vec<String> = session
                    .screen
                    .lock()
                    .map(|s| s.text_lines(n))
                    .unwrap_or_default();
                IpcResponse::ok(serde_json::json!({ "id": id, "lines": output }))
            } else {
                IpcResponse::error(format!("no terminal with id {id}"))
            }
        }

        IpcRequest::SendCommand { id, command } => {
            if let Some(session) = mgr.get_mut(id) {
                let mut data = command.into_bytes();
                data.push(b'\r');
                match session.write(&data) {
                    Ok(_) => IpcResponse::ok(serde_json::json!({ "sent": true })),
                    Err(e) => IpcResponse::error(e.to_string()),
                }
            } else {
                IpcResponse::error(format!("no terminal with id {id}"))
            }
        }

        IpcRequest::GetTerminalInfo { id } => {
            if let Some(session) = mgr.get_mut(id) {
                IpcResponse::ok(serde_json::json!({
                    "id": session.id,
                    "label": session.label,
                    "cols": session.cols,
                    "rows": session.rows,
                }))
            } else {
                IpcResponse::error(format!("no terminal with id {id}"))
            }
        }

        IpcRequest::CreateTerminal { label, command: _ } => {
            match mgr.create(label, 80, 24) {
                Ok(id) => IpcResponse::ok(serde_json::json!({ "id": id })),
                Err(e) => IpcResponse::error(e.to_string()),
            }
        }
    }
}

/// Returns the session ID stored in the filesystem for discovery.
pub fn write_session_file(session_id: &str) -> Result<()> {
    let dir = dirs::runtime_dir()
        .or_else(|| dirs::data_local_dir())
        .unwrap_or_else(|| std::path::PathBuf::from("/tmp"))
        .join("holocron");
    std::fs::create_dir_all(&dir)?;
    std::fs::write(dir.join("session"), session_id)?;
    Ok(())
}

pub fn read_session_id() -> Result<String> {
    let dir = dirs::runtime_dir()
        .or_else(|| dirs::data_local_dir())
        .unwrap_or_else(|| std::path::PathBuf::from("/tmp"))
        .join("holocron");
    Ok(std::fs::read_to_string(dir.join("session"))?.trim().to_string())
}

pub fn cleanup_session_file() {
    if let Some(dir) = dirs::runtime_dir().or_else(|| dirs::data_local_dir()) {
        let _ = std::fs::remove_file(dir.join("holocron").join("session"));
    }
}
