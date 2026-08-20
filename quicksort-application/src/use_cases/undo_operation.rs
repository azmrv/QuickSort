use crate::domain::{
    Operation, OperationId, OperationState, DomainEvent, WindowsPath, DomainError,
};
use crate::application::ports::{
    FileSystemPort, ConflictResolverPort, OperationRepositoryPort, IdGeneratorPort, EventPublisherPort,
};
use tracing::{info, error};

// Placeholder for the Undo Use Case structure
pub struct UndoOperationUseCase {
    operation_repo: Box<dyn OperationRepositoryPort>,
    event_publisher: Box<dyn EventPublisherPort>,
}

impl UndoOperationUseCase {
    pub fn new(
        operation_repo: Box<dyn OperationRepositoryPort>,
        event_publisher: Box<dyn EventPublisherPort>,
    ) -> Self {
        Self {
            operation_repo,
            event_publisher,
        }
    }

    /// Reverses a previously executed operation by transitioning the Operation state to Undone.
    pub async fn undo(&self, operation_id: OperationId) -> Result<(), DomainError> {
        info!("Attempting to undo operation ID: {}", operation_id.as_str());

        // 1. Fetch the current operation from the repository
        let mut operation = self.operation_repo.get_operation(&operation_id).await
            .map_err(|e| DomainError::OperationNotFound(format!("Failed to retrieve operation: {}", e)))?;

        info!("Retrieved Operation ID {} with state: {:?}", operation.id.as_str(), operation.state);

        // 2. Validate that the operation is in a reversible state
        if !matches!(operation.state, OperationState::Completed { .. }) {
            return Err(DomainError::InvalidStateTransition("Cannot undo non-completed or already undone operations.".to_string()));
        }

        info!("Operation {} is marked for undo.", operation.id.as_str());

        // 3. Execute the rollback logic (delegated to Domain Entity)
        match operation.mark_undone(std::time::SystemTime::now()) {
            Ok(_) => {
                info!("Operation {} successfully transitioned to Undone state.", operation.id.as_str());
                
                // 4. Publish the success event
                let undo_event = DomainEvent::OperationUndone {
                    operation_id: operation.id.clone(),
                };
                self.event_publisher.publish(undo_event).await;
                Ok(())
            }
            Err(e) => {
                error!("Failed to mark operation {} as undone: {:?}", operation.id.as_str(), e);
                Err(e)
            }
        }
    }
}