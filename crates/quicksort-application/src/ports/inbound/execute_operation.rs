//! Port for executing file operations.

use async_trait::async_trait;
use crate::dtos::{OperationCommand, OperationResult};
use crate::errors::UseCaseError;

/// Implemented by ExecuteOperationUseCase.
#[async_trait]
pub trait ExecuteOperation: Send + Sync {
    /// Execute the given operation command.
    /// Returns the result and a collection of domain events.
    async fn execute(&self, command: OperationCommand) -> Result<OperationResult, UseCaseError>;
}