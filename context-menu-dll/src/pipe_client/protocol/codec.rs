//! Serialization codec: converts commands to bytes.

use serde::{Deserialize, Serialize};
use crate::pipe_client::error::PipeError;
use super::envelope::MessageEnvelope;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandMessage {
    pub command: CommandKind,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandKind {
    ExecuteOperation,
    Ping,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteOperationData {
    pub operation_type: OperationType,
    pub source_paths: Vec<String>,
    pub target_folder_id: Option<String>,
    pub overwrite_policy: OverwritePolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationType {
    Move,
    Copy,
    Delete,
    Rename,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OverwritePolicy {
    Skip,
    Overwrite,
    AutoRename,
    Ask,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMessage {
    pub status: ResponseStatus,
    pub message: String,
    pub operation_id: Option<String>,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResponseStatus {
    Ok,
    Error,
    Pending,
}

pub struct Codec;

impl Codec {
    pub fn encode_command(command: CommandKind, data: impl Serialize) -> Result<Vec<u8>, PipeError> {
        let msg = CommandMessage {
            command,
            data: serde_json::to_value(data)?,
        };
        let json = serde_json::to_vec(&msg)?;
        let envelope = MessageEnvelope::new(json)?;
        Ok(envelope.encode())
    }

    pub fn decode_response(data: &[u8]) -> Result<ResponseMessage, PipeError> {
        let envelope = MessageEnvelope::decode(data)?;
        let response: ResponseMessage = serde_json::from_slice(&envelope.payload)?;
        Ok(response)
    }
}