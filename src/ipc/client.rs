use crate::ipc::protocol::{IpcRequest, IpcResponse};
use anyhow::{Context, Result};
use interprocess::local_socket::{
    tokio::prelude::*, GenericNamespaced, ToNsName,
};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

pub struct IpcClient {
    session_id: String,
}

impl IpcClient {
    pub fn new(session_id: String) -> Self {
        Self { session_id }
    }

    /// Probes whether the session's IPC socket is actually reachable,
    /// without sending a request. Used to detect a stale session file
    /// left behind by a crashed TUI or headless daemon.
    pub async fn is_alive(&self) -> bool {
        let Ok(name) = crate::ipc::server::socket_name(&self.session_id)
            .to_ns_name::<GenericNamespaced>()
        else {
            return false;
        };

        interprocess::local_socket::tokio::Stream::connect(name)
            .await
            .is_ok()
    }

    pub async fn send(&self, req: IpcRequest) -> Result<IpcResponse> {
        let name = crate::ipc::server::socket_name(&self.session_id)
            .to_ns_name::<GenericNamespaced>()?;

        let conn = interprocess::local_socket::tokio::Stream::connect(name)
            .await
            .context("failed to connect to holocron IPC socket — is holocron running?")?;

        let (reader, mut writer) = tokio::io::split(conn);

        let mut payload = serde_json::to_string(&req)?;
        payload.push('\n');
        writer.write_all(payload.as_bytes()).await?;

        let mut lines = BufReader::new(reader).lines();
        let line = lines
            .next_line()
            .await?
            .context("IPC connection closed without response")?;

        serde_json::from_str::<IpcResponse>(&line).context("invalid IPC response")
    }
}
