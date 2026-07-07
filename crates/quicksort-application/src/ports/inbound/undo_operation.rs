// //! Port for undoing a completed operation.
// 
// use async_trait::async_trait;
// use crate::dtos::OperationResult;
// use crate::errors::UseCaseError;
// use quicksort_domain::OperationId;
// 
// #[async_trait]
// pub trait UndoOperation: Send + Sync {
//     /// Reverse the effects of the operation identified by `operation_id`.
//     async fn undo(&self, operation_id: OperationId) -> Result<OperationResult, UseCaseError>;
// }