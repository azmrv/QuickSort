//! Use case for retrieving operation history.

use crate::errors::UseCaseError;
use crate::ports::inbound::GetOperationHistory;
use crate::ports::outbound::OperationRepository;
use async_trait::async_trait;
use quicksort_domain::Operation;

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
}
