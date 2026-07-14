//! Domain events are immutable records of business facts.
//!
//! Events are immutable records of domain facts.
//! They are used to track changes in system state and audit operations.

use serde::{Serialize, Deserialize};
use crate::value_objects::{OperationId, FolderId};
use crate::entities::OperationType;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DomainEvent {
    OperationStarted {
        operation_id: OperationId,
        op_type: OperationType,
    },
    OperationCompleted {
        operation_id: OperationId,
        files: u32,
        bytes: u64,
    },
    OperationFailed {
        operation_id: OperationId,
        reason: String,
    },
    OperationUndone {
        operation_id: OperationId,
    },
    FolderAdded {
        folder_id: FolderId,
        name: String,
        path: String,
    },
    FolderRemoved {
        folder_id: FolderId,
        name: String,
    },
    FolderRenamed {
        folder_id: FolderId,
        old_name: String,
        new_name: String,
    },
}