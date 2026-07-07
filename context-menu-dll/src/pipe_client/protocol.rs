//! IPC protocol definitions (v1: length-prefixed JSON).

use serde::{Deserialize, Serialize};

pub const PROTOCOL_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipeCommand {
    pub version: u32,
    pub action: PipeAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PipeAction {
    ExecuteOperation { command: OperationCommand },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationCommand {
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