//! Outbound port: OperationRepository
//!
//! Defines the interface that infrastructure must implement to persist
//! and retrieve `Operation` aggregates. This port is used by
//! `ExecuteOperationUseCase` and `UndoOperationUseCase`.

use async_trait::async_trait;
use quicksort_domain::Operation;
use crate::errors::UseCaseError;

/// Persistence port for operations.
///
/// All methods return `UseCaseError` to keep the Application layer
/// independent of infrastructure-specific error types.
#[async_trait]
pub trait OperationRepository: Send + Sync {
    /// Persist a new or updated operation.
    async fn save(&self, operation: &Operation) -> Result<(), UseCaseError>;

    /// Retrieve an operation by its unique identifier.
    /// Returns `None` if the operation is not found.
    async fn find_by_id(
        &self,
        id: &quicksort_domain::OperationId,
    ) -> Result<Option<Operation>, UseCaseError>;

    /// Retrieve all stored operations.
    async fn load_all(&self) -> Result<Vec<Operation>, UseCaseError>;

    /// Retrieve the most recent operation, if any.
    async fn load_last(&self) -> Result<Option<Operation>, UseCaseError>;

    /// Delete all stored operations (e.g., for testing or data reset).
    async fn delete_all(&self) -> Result<(), UseCaseError>;
}

// OLD: original trait definition (kept for reference)
// #[async_trait::async_trait]
// pub trait OperationRepository: Send + Sync {
//     async fn save(&self, operation: &Operation) -> Result<()>;
//     async fn load_all(&self) -> Result<Vec<Operation>>;
//     async fn load_last(&self) -> Result<Option<Operation>>;
//     async fn update_status(&self, id: &str, status: OperationStatus) -> Result<()>;
//     async fn delete_all(&self) -> Result<()>;
// }
//
// NEW: redesigned with explicit OperationId, OperationStatus removed
// (status is managed inside the Operation aggregate), and proper
// error type (UseCaseError) instead of anyhow::Result.
//
// The old `update_status` method was removed because status transitions
// are encapsulated within the `Operation` domain entity (start, complete,
// fail, mark_undone). Infrastructure should only save/load full aggregates.