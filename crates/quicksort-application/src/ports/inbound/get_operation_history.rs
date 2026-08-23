//! Inbound port for retrieving operation history.
//!
//! This port provides access to the history of all file operations,
//! enabling the frontend to display operation logs and support undo functionality.

use async_trait::async_trait;
use quicksort_domain::Operation;

use crate::errors::UseCaseError;

/// Trait for retrieving operation history.
#[async_trait]
pub trait GetOperationHistory: Send + Sync {
    /// Returns all stored operations, sorted by creation time (newest first).
    async fn get_all_operations(&self) -> Result<Vec<Operation>, UseCaseError>;
}
