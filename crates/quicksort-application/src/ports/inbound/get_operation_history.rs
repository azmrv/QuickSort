//! Inbound port for retrieving operation history.
//!
//! This port provides access to the history of all file operations,
//! enabling the frontend to display operation logs and support undo functionality.

use async_trait::async_trait;
use quicksort_domain::{Operation, OperationId};

use crate::errors::UseCaseError;

/// Trait for retrieving and pruning operation history.
#[async_trait]
pub trait GetOperationHistory: Send + Sync {
    /// Returns all stored operations, sorted by creation time (newest first).
    async fn get_all_operations(&self) -> Result<Vec<Operation>, UseCaseError>;

    /// Deletes a single operation by its identifier.
    ///
    /// This lets the UI remove one history entry (e.g. to drop a mistaken or
    /// no-longer-relevant record) without wiping the whole audit trail.
    async fn delete_operation(&self, id: OperationId) -> Result<(), UseCaseError>;

    /// Deletes every stored operation, leaving the history empty.
    async fn clear_history(&self) -> Result<(), UseCaseError>;
}
