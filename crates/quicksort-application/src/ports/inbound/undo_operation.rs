//! Port for undoing a completed operation.
//! Provides the interface for reverting (undoing) previously executed operations.

use async_trait::async_trait;
use crate::dtos::OperationResult;
use crate::errors::UseCaseError;
use quicksort_domain::{Operation, OperationId, OperationState};

/// Trait for undoing operations.
/// This port allows rolling back a previously completed operation, restoring files to their original state.
#[async_trait]
pub trait UndoOperation: Send + Sync {
    /// Undoes the operation with the specified ID.
    ///
    /// # Arguments
    /// * `operation_id` - Unique identifier of the operation to undo
    ///
    /// # Returns
    /// Operation result containing:
    /// - number of restored files
    /// - bytes processed during undo
    ///
    /// # Errors
    /// * `UseCaseError::OperationNotFound` - operation not found in history
    /// * `UseCaseError::InvalidState` - operation cannot be undone (not completed or already undone)
    /// * `UseCaseError::Internal` - internal error during undo execution
    async fn undo(&self, operation_id: OperationId) -> Result<OperationResult, UseCaseError>;
}