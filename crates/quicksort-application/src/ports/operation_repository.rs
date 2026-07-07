// // quicksort-application/src/ports/operation_repository.rs
//
// use crate::domain::Operation;
// use anyhow::Result;
//
// #[async_trait::async_trait]
// pub trait OperationRepository: Send + Sync {
//     async fn save(&self, operation: &Operation) -> Result<()>;
//     async fn load_all(&self) -> Result<Vec<Operation>>;
//     async fn load_last(&self) -> Result<Option<Operation>>;
//     async fn update_status(&self, id: &str, status: OperationStatus) -> Result<()>;
//     async fn delete_all(&self) -> Result<()>;
// }