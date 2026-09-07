//! Use case for retrieving and pruning operation history.

use crate::errors::UseCaseError;
use crate::ports::inbound::GetOperationHistory;
use crate::ports::outbound::OperationRepository;
use async_trait::async_trait;
use quicksort_domain::{Operation, OperationId};

pub struct GetOperationHistoryUseCase {
    operation_repository: Box<dyn OperationRepository>,
}

impl GetOperationHistoryUseCase {
    pub fn new(operation_repository: Box<dyn OperationRepository>) -> Self {
        Self {
            operation_repository,
        }
    }
}

#[async_trait]
impl GetOperationHistory for GetOperationHistoryUseCase {
    async fn get_all_operations(&self) -> Result<Vec<Operation>, UseCaseError> {
        let mut operations = self
            .operation_repository
            .load_all()
            .await
            .map_err(|e| UseCaseError::RepositoryError(e.to_string()))?;

        operations.sort_by_key(|op| std::cmp::Reverse(op.created_at));
        Ok(operations)
    }

    async fn delete_operation(&self, id: OperationId) -> Result<(), UseCaseError> {
        self.operation_repository
            .delete(&id)
            .await
            .map_err(|e| UseCaseError::RepositoryError(e.to_string()))
    }

    async fn clear_history(&self) -> Result<(), UseCaseError> {
        self.operation_repository
            .clear()
            .await
            .map_err(|e| UseCaseError::RepositoryError(e.to_string()))
    }
}
