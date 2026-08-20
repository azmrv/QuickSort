use std::time::SystemTime;
// Assuming these types are defined in quicksort-domain/src/dtos/operation_command.rs or similar
use crate::domain::{Operation, OperationState, DomainEvent, WindowsPath, OperationId};

#[derive(Debug, Clone)]
pub struct Operation {
    pub id: OperationId,
    pub op_type: OperationType,
    pub source_path: Option<WindowsPath>,
    pub target_path: Option<WindowsPath>,
    pub original_name: Option<String>, // New field for Rename tracking
    pub state: OperationState,
    pub updated_at: Option<SystemTime>,
    pub events: Vec<DomainEvent>,
}

impl Operation {
    // Existing methods...

    /// Transitions the operation to Undone state.
    pub fn mark_undone(&mut self, now: SystemTime) -> Result<(), crate::domain::DomainError> {
        if !matches!(self.state, OperationState::Completed { .. }) {
            return Err(crate::domain::DomainError::InvalidStateTransition);
        }
        self.state = OperationState::Undone;
        self.updated_at = Some(now);
        self.events.push(DomainEvent::OperationUndone {
            operation_id: self.id.clone(),
        });
        Ok(())
    }

    // Placeholder for future methods to store operation-specific data...
}