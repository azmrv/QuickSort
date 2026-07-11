// quicksort-domain/src/operation.rs

use crate::{FolderId, WindowsPath};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OperationType {
    Move,
    Copy,
    Delete,
    Rename,
    // позже можно добавить CreateFolder, etc.
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OperationStatus {
    Pending,
    Completed,
    Failed(String),
    Undone,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    pub id: String, // UUID
    pub op_type: OperationType,
    pub status: OperationStatus,

    // Для отката храним и source, и target
    pub source: WindowsPath,
    pub target: Option<WindowsPath>, // для Move/Copy/Rename нужен target

    // Для Rename – старое имя и новое
    pub old_name: Option<String>,
    pub new_name: Option<String>,

    pub timestamp: SystemTime,
    pub error: Option<String>,
}

impl Operation {
    pub fn new_move(source: WindowsPath, target: WindowsPath) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(), // TODO: Fix UUID generation for v7/v4
            op_type: OperationType::Move,
            status: OperationStatus::Pending,
            source,
            target: Some(target),
            old_name: None,
            new_name: None,
            timestamp: SystemTime::now(),
            error: None,
        }
    }

    pub fn new_copy(source: WindowsPath, target: WindowsPath) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(), // TODO: Fix UUID generation for v7/v4
            op_type: OperationType::Copy,
            status: OperationStatus::Pending,
            source,
            target: Some(target),
            old_name: None,
            new_name: None,
            timestamp: SystemTime::now(),
            error: None,
        }
    }

    pub fn new_delete(source: WindowsPath) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(), // TODO: Fix UUID generation for v7/v4
            op_type: OperationType::Delete,
            status: OperationStatus::Pending,
            source,
            target: None,
            old_name: None,
            new_name: None,
            timestamp: SystemTime::now(),
            error: None,
        }
    }

    pub fn new_rename(source: WindowsPath, target: WindowsPath, old_name: String, new_name: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(), // TODO: Fix UUID generation for v7/v4
            op_type: OperationType::Rename,
            status: OperationStatus::Pending,
            source,
            target: Some(target),
            old_name: Some(old_name),
            new_name: Some(new_name),
            timestamp: SystemTime::now(),
            error: None,
        }
    }
}