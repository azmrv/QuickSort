//! Inbound port for executing file operations.
//!
//! This port defines the contract that all adapters (Tauri, CLI, IPC)
//! use to perform Move, Copy, Delete, and Rename operations.
//! It is implemented by `ExecuteOperationUseCase`.

use async_trait::async_trait;
use crate::dtos::{OperationCommand, OperationResult};
use crate::errors::UseCaseError;

/// Inbound port for executing a file operation.
///
/// # Usage
/// Adapters create an `OperationCommand` from user input (or IPC) and
/// pass it to this method. The port returns an `OperationResult` containing
/// the operation ID, final state, and processing statistics.
///
/// # Errors
/// Returns `UseCaseError` if validation fails, the operation cannot be
/// completed, or the underlying infrastructure reports an error.
#[async_trait]
pub trait ExecuteOperation: Send + Sync {
    /// Executes the given operation command.
    ///
    /// # Parameters
    /// - `command` – Contains the operation type, source paths, target folder,
    ///   and overwrite policy.
    ///
    /// # Returns
    /// On success, returns an `OperationResult` with the operation's details.
    /// On failure, returns a `UseCaseError` describing the problem.
    async fn execute(&self, command: OperationCommand) -> Result<OperationResult, UseCaseError>;
}