//! Domain events – business facts emitted by aggregates.

use crate::value_objects::{OperationId};
use crate::entities::OperationType;

#[derive(Debug, Clone, PartialEq, Eq)]
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
}