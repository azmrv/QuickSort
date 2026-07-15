//! Outbound port for operation history storage.
//!
//! This port defines the interface for persisting and retrieving
//! `Operation` aggregates. It is used by `ExecuteOperationUseCase`
//! and `UndoOperationUseCase` to record operation history for
//! auditing, undo functionality, and status tracking.
//!
//! # Design Decision
//! The repository operates on domain aggregates (`Operation`) rather
//! than DTOs. This keeps the Application layer independent of
//! serialization formats and allows the domain model to evolve
//! without affecting persistence code.

use async_trait::async_trait;
use quicksort_domain::{Operation, OperationId};
use crate::errors::UseCaseError;

/// Persistence port for operation aggregates.
///
/// All methods return `UseCaseError` to insulate the Application
/// layer from infrastructure-specific error types (e.g., I/O errors,
/// serialization failures).
#[async_trait]
pub trait OperationRepository: Send + Sync {
    /// Finds an operation by its unique identifier.
    ///
    /// # Returns
    /// `Some(Operation)` if found, `None` otherwise.
    ///
    /// # Errors
    /// Returns `RepositoryError` if the underlying storage fails
    /// (e.g., file read error, database connection lost).
    async fn find_by_id(&self, id: &OperationId) -> Result<Option<Operation>, UseCaseError>;

    /// Persists an operation (creates a new record or updates an existing one).
    ///
    /// # Usage
    /// Called by Use Cases after an operation completes or is undone.
    ///
    /// # Errors
    /// Returns `RepositoryError` if the save operation fails.
    async fn save(&self, operation: &Operation) -> Result<(), UseCaseError>;

    /// Deletes an operation by its unique identifier.
    ///
    /// If the operation does not exist, the operation is idempotent
    /// and does not return an error.
    ///
    /// # Errors
    /// Returns `RepositoryError` if the storage operation fails.
    async fn delete(&self, id: &OperationId) -> Result<(), UseCaseError>;

    /// Loads all stored operations.
    ///
    /// # Returns
    /// A vector of all operations, or an empty vector if none exist.
    ///
    /// # Errors
    /// Returns `RepositoryError` if the underlying storage cannot be read.
    async fn load_all(&self) -> Result<Vec<Operation>, UseCaseError>;
}