use async_trait::async_trait;
use crate::dtos::OperationCommand;
use crate::errors::UseCaseError;

#[async_trait]
pub trait ConflictResolver: Send + Sync {
    async fn resolve(&self, command: OperationCommand) -> Result<OperationCommand, UseCaseError>;
}