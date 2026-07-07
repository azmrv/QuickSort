//! Outbound port for operation history storage.

use async_trait::async_trait;
use quicksort_domain::{Operation, OperationId};
use crate::errors::UseCaseError;

#[async_trait]
pub trait OperationRepository: Send + Sync {
    async fn find_by_id(&self, id: &OperationId) -> Result<Option<Operation>, UseCaseError>;
    async fn save(&self, operation: &Operation) -> Result<(), UseCaseError>;
    async fn delete(&self, id: &OperationId) -> Result<(), UseCaseError>;
    async fn load_all(&self) -> Result<Vec<Operation>, UseCaseError>;
}