//! Result DTO returned by ExecuteOperation and UndoOperation.

use quicksort_domain::{OperationId, OperationState};

/// Result of a completed or failed operation.
#[derive(Debug, Clone)]
pub struct OperationResult {
    pub operation_id: OperationId,
    pub state: OperationState,
    pub processed_files: u32,
    pub bytes_moved: u64,
    // In the future, we can add more fields:
    // pub failed_files: Vec<WindowsPath>,
    // pub errors: Vec<String>,
}