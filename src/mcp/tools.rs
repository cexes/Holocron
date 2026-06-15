use crate::ipc::{client::IpcClient, protocol::IpcRequest};
use anyhow::Result;
use rmcp::model::{CallToolResult, Content};
use uuid::Uuid;

pub struct TerminalTools {
    client: IpcClient,
}

impl TerminalTools {
    pub fn new(session_id: String) -> Self {
        Self { client: IpcClient::new(session_id) }
    }

    pub async fn list_terminals(&self) -> Result<CallToolResult> {
        let resp = self.client.send(IpcRequest::ListTerminals).await?;
        Ok(resp_to_tool_result(resp))
    }

    pub async fn read_terminal(&self, id: Uuid, lines: Option<usize>) -> Result<CallToolResult> {
        let resp = self.client.send(IpcRequest::ReadTerminal { id, lines }).await?;
        Ok(resp_to_tool_result(resp))
    }

    pub async fn send_command(&self, id: Uuid, command: String) -> Result<CallToolResult> {
        let resp = self.client.send(IpcRequest::SendCommand { id, command }).await?;
        Ok(resp_to_tool_result(resp))
    }

    pub async fn get_terminal_info(&self, id: Uuid) -> Result<CallToolResult> {
        let resp = self.client.send(IpcRequest::GetTerminalInfo { id }).await?;
        Ok(resp_to_tool_result(resp))
    }

    pub async fn create_terminal(
        &self,
        label: String,
        command: Option<String>,
    ) -> Result<CallToolResult> {
        let resp = self
            .client
            .send(IpcRequest::CreateTerminal { label, command })
            .await?;
        Ok(resp_to_tool_result(resp))
    }
}

fn resp_to_tool_result(resp: crate::ipc::protocol::IpcResponse) -> CallToolResult {
    use crate::ipc::protocol::IpcResponse;
    match resp {
        IpcResponse::Ok { payload } => CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&payload).unwrap_or_default(),
        )]),
        IpcResponse::Error { message } => CallToolResult::error(vec![Content::text(message)]),
    }
}

