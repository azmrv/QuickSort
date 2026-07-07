// //! UndoOperationUseCase - reverts a completed operation.
//
// use async_trait::async_trait;
// use std::sync::Arc;
// use quicksort_domain::{OperationId, OperationState, OperationType, WindowsPath};
// use crate::dtos::OperationResult;
// use crate::errors::UseCaseError;
// use crate::ports::outbound::{OperationRepository, FileSystem, Clock};
// use crate::ports::inbound::UndoOperation;
//
// pub struct UndoOperationUseCase {
//     operation_repo: Arc<dyn OperationRepository>,
//     file_system: Arc<dyn FileSystem>,
//     clock: Arc<dyn Clock>,
// }
//
// impl UndoOperationUseCase {
//     pub fn new(
//         operation_repo: Arc<dyn OperationRepository>,
//         file_system: Arc<dyn FileSystem>,
//         clock: Arc<dyn Clock>,
//     ) -> Self {
//         Self { operation_repo, file_system, clock }
//     }
// }
//
// #[async_trait]
// impl UndoOperation for UndoOperationUseCase {
//     async fn undo(&self, operation_id: OperationId) -> Result<OperationResult, UseCaseError> {
//         // 1. Load operation
//         let mut op = self.operation_repo
//             .find_by_id(&operation_id)
//             .await
//             .map_err(|e| UseCaseError::RepositoryError(e.to_string()))?
//             .ok_or_else(|| UseCaseError::OperationNotFound(operation_id.clone()))?;
//
//         // 2. Check if operation is completed and not undone
//         if op.state != OperationState::Completed {
//             return Err(UseCaseError::UndoNotPossible(
//                 "Only completed operations can be undone".to_string(),
//             ));
//         }
//
//         // 3. Perform undo based on operation type
//         let result = match op.operation_type {
//             OperationType::Move => self.undo_move(&op).await,
//             OperationType::Copy => self.undo_copy(&op).await,
//             OperationType::Delete => self.undo_delete(&op).await,
//             OperationType::Rename => self.undo_rename(&op).await,
//         };
//
//         // 4. If successful, update operation state to Undone
//         match result {
//             Ok(()) => {
//                 // Update state in repository
//                 // We need to mutate the op and save
//                 op.state = OperationState::Undone;
//                 op.updated_at = self.clock.now();
//                 self.operation_repo
//                     .save(&op)
//                     .await
//                     .map_err(|e| UseCaseError::RepositoryError(e.to_string()))?;
//
//                 Ok(OperationResult {
//                     operation_id: op.id.clone(),
//                     state: OperationState::Undone,
//                     processed_files: 1,
//                     bytes_moved: 0,
//                 })
//             }
//             Err(e) => {
//                 // Could update state to Failed here if needed
//                 Err(e)
//             }
//         }
//     }
// }
//
// // ===== Helper methods =====
//
// impl UndoOperationUseCase {
//     async fn undo_move(&self, op: &quicksort_domain::Operation) -> Result<(), UseCaseError> {
//         // We need to reverse the move: move from target back to source
//         // But op.source_paths is Vec<WindowsPath>, and op.target_folder_path is Option<WindowsPath>
//         // We need to reconstruct original and target paths
//         let source_path = op.source_paths.first()
//             .ok_or_else(|| UseCaseError::UndoNotPossible("No source path found".to_string()))?;
//         let target_folder = op.target_folder_path.as_ref()
//             .ok_or_else(|| UseCaseError::UndoNotPossible("No target folder for Move".to_string()))?;
//
//         // Determine destination file name (use the same file name as source)
//         let file_name = source_path.as_str().split('\\').last()
//             .ok_or_else(|| UseCaseError::UndoNotPossible("Invalid source path".to_string()))?;
//         let target_path = WindowsPath::new(&format!("{}\\{}", target_folder.as_str(), file_name))
//             .map_err(|_| UseCaseError::UndoNotPossible("Invalid target path".to_string()))?;
//
//         // Check if file exists at target
//         if !self.file_system.exists(&target_path).await
//             .map_err(|e| UseCaseError::FileSystemError(e.to_string()))?
//         {
//             return Err(UseCaseError::UndoNotPossible(
//                 "File no longer exists at target location".to_string(),
//             ));
//         }
//
//         // Move back to source
//         self.file_system.rename(&target_path, source_path).await
//             .map_err(|e| UseCaseError::FileSystemError(e.to_string()))?;
//
//         Ok(())
//     }
//
//     async fn undo_copy(&self, op: &quicksort_domain::Operation) -> Result<(), UseCaseError> {
//         let target_folder = op.target_folder_path.as_ref()
//             .ok_or_else(|| UseCaseError::UndoNotPossible("No target folder for Copy".to_string()))?;
//         let source_path = op.source_paths.first()
//             .ok_or_else(|| UseCaseError::UndoNotPossible("No source path found".to_string()))?;
//         let file_name = source_path.as_str().split('\\').last()
//             .ok_or_else(|| UseCaseError::UndoNotPossible("Invalid source path".to_string()))?;
//         let target_path = WindowsPath::new(&format!("{}\\{}", target_folder.as_str(), file_name))
//             .map_err(|_| UseCaseError::UndoNotPossible("Invalid target path".to_string()))?;
//
//         if self.file_system.exists(&target_path).await
//             .map_err(|e| UseCaseError::FileSystemError(e.to_string()))?
//         {
//             self.file_system.remove(&target_path).await
//                 .map_err(|e| UseCaseError::FileSystemError(e.to_string()))?;
//         }
//
//         Ok(())
//     }
//
//     async fn undo_delete(&self, op: &quicksort_domain::Operation) -> Result<(), UseCaseError> {
//         // For now, we don't support undo of delete without a trash mechanism
//         Err(UseCaseError::UndoNotPossible(
//             "Undo of Delete operation is not yet supported".to_string(),
//         ))
//     }
//
//     async fn undo_rename(&self, op: &quicksort_domain::Operation) -> Result<(), UseCaseError> {
//         // Rename: we need old name and new name
//         // Since we don't have old_name/new_name fields, we can derive from source_paths and maybe a separate field
//         // For now, let's assume we store old path in source_paths[0] and new path in some other field.
//         // But we don't have that. We'll need to add fields for old and new names.
//         // As a workaround, we can use the fact that source_paths[0] is the old path, and we need the new path from somewhere.
//         // Let's require that op.target_folder_path holds the new full path.
//         Err(UseCaseError::UndoNotPossible(
//             "Undo of Rename requires old_name/new_name fields in Operation".to_string(),
//         ))
//     }
// }