//! Default conflict resolver that passes commands through unchanged.

use async_trait::async_trait;
use quicksort_application::ports::outbound::ConflictResolver;
use quicksort_application::dtos::OperationCommand;
use quicksort_application::errors::UseCaseError;

pub struct DefaultConflictResolver;

#[async_trait]
impl ConflictResolver for DefaultConflictResolver {
    async fn resolve(&self, command: OperationCommand) -> Result<OperationCommand, UseCaseError> {
        // For now, just pass the command through without modification.
        // In the future, this could apply renaming, skipping, or overwriting.
        Ok(command)
    }
}