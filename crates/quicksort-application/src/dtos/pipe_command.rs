//! DTO for IPC communication between DLL and Tauri.

use serde::{Deserialize, Serialize};
use crate::dtos::OperationCommand;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipeCommand {
    pub version: u32,
    pub action: PipeAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PipeAction {
    ExecuteOperation {
        command: OperationCommand,
    },
    // TODO: Add other actions if needed (e.g., GetFolders, etc.)
}