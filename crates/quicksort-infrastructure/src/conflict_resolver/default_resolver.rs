use quicksort_application::ports::outbound::ConflictResolver;
use quicksort_application::dtos::OperationCommand;
use quicksort_application::errors::UseCaseError;

pub struct DefaultConflictResolver;

impl ConflictResolver for DefaultConflictResolver {
    async fn resolve(&self, command: OperationCommand) -> Result<OperationCommand, UseCaseError> {
        // For now, just return the command unchanged.
        // In real implementation, we could add more sophisticated logic.
        Ok(command)
    }
}