//! UndoOperationUseCase - reverts a completed operation.

use async_trait::async_trait;
use std::sync::Arc;
use quicksort_domain::{OperationId, OperationState};
use crate::dtos::OperationResult;
use crate::errors::UseCaseError;
use crate::ports::outbound::{OperationRepository, FileSystem, Clock};
use crate::ports::inbound::UndoOperation;

pub struct UndoOperationUseCase {
    operation_repo: Arc<dyn OperationRepository>,
    file_system: Arc<dyn FileSystem>,
    clock: Arc<dyn Clock>,
}

impl UndoOperationUseCase {
    pub fn new(
        operation_repo: Arc<dyn OperationRepository>,
        file_system: Arc<dyn FileSystem>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self { operation_repo, file_system, clock }
    }
}

#[async_trait]
impl UndoOperation for UndoOperationUseCase {
    async fn undo(&self, operation_id: OperationId) -> Result<OperationResult, UseCaseError> {
        // TODO: implement undo logic
        Err(UseCaseError::UndoNotPossible("Not implemented".to_string()))
    }
}