//! Inbound port for undoing a completed operation.
//!
//! This port is part of the Application Layer's inbound interface.
//! It is implemented by `UndoOperationUseCase` and called by adapters
//! (Tauri commands, IPC handlers) to revert previously executed operations.

use async_trait::async_trait;
use crate::dtos::OperationResult;
use crate::errors::UseCaseError;
use quicksort_domain::{Operation, OperationId, OperationState};

/// Trait for undoing operations.
///
/// This port allows rolling back a previously completed operation,
/// restoring files to their original state.
///
/// # How It Works
/// The implementing use case:
/// 1. Loads the operation from the repository.
/// 2. Checks that the operation is in `Completed` state.
/// 3. Performs the reverse file operation (Move back, Delete copy, etc.).
/// 4. Marks the operation as `Undone` using the domain aggregate's method.
/// 5. Persists the updated operation.
///
/// # Error Conditions
/// - `OperationNotFound` – the given ID does not correspond to any stored operation.
/// - `UndoNotPossible` – the operation is not in a state that can be undone
///   (e.g., already undone, never completed, or in progress).
/// - `FileSystemError` – a file operation failed (file missing, permission denied, etc.).
/// - `RepositoryError` – could not load or save the operation.
#[async_trait]
pub trait UndoOperation: Send + Sync {
    /// Undoes the operation with the specified ID.
    ///
    /// # Arguments
    /// * `operation_id` - Unique identifier of the operation to undo.
    ///
    /// # Returns
    /// An `OperationResult` containing the updated operation state and
    /// processing statistics (number of files restored, bytes processed).
    ///
    /// # Errors
    /// Returns `UseCaseError` if the operation cannot be undone or the
    /// underlying infrastructure reports a failure.
    async fn undo(&self, operation_id: OperationId) -> Result<OperationResult, UseCaseError>;
}