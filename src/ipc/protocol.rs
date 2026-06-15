use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum IpcRequest {
    ListTerminals,
    ReadTerminal { id: Uuid, lines: Option<usize> },
    SendCommand { id: Uuid, command: String },
    GetTerminalInfo { id: Uuid },
    CreateTerminal { label: String, command: Option<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum IpcResponse {
    Ok { payload: serde_json::Value },
    Error { message: String },
}

impl IpcResponse {
    pub fn ok(payload: impl Serialize) -> Self {
        Self::Ok {
            payload: serde_json::to_value(payload).unwrap_or(serde_json::Value::Null),
        }
    }

    pub fn error(msg: impl Into<String>) -> Self {
        Self::Error { message: msg.into() }
    }
}
