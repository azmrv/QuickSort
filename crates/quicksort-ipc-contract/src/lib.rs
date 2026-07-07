//! Единый IPC-протокол для QuickSort.
//! Используется DLL (клиент) и Tauri (сервер).

use serde::{Deserialize, Serialize};

pub const PROTOCOL_VERSION: u16 = 1;
pub const MAGIC: u32 = 0x51535452; // "QSTR"

/// Команда от клиента к серверу.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum CommandMessage {
    ExecuteOperation(ExecuteOperationData),
    Ping,
}

/// Данные для выполнения операции.
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

/// Ответ сервера клиенту.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMessage {
    pub status: ResponseStatus,
    pub message: String,
    pub operation_id: Option<String>,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ResponseStatus {
    Ok,
    Error,
    Pending,
}