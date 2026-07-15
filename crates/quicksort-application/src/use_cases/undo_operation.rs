//! UndoOperationUseCase - reverts a completed operation.
//!
//! # Design Decisions
//! - Uses `Operation::mark_undone()` instead of directly modifying state to
//!   preserve domain invariants and trigger domain events.
//! - All file-system interactions go through the `FileSystem` port, keeping
//!   the use case testable and independent of the actual file system.
//! - Each undo strategy is isolated in a private helper method for readability.

use async_trait::async_trait;
use std::sync::Arc;
use quicksort_domain::{
    Operation, OperationId, OperationType, OperationState, WindowsPath,
};
use crate::dtos::OperationResult;
use crate::errors::UseCaseError;
use crate::ports::outbound::{OperationRepository, FileSystem, Clock};
use crate::ports::inbound::UndoOperation;

/// Use case implementation for undoing operations.
///
/// It loads the operation from the repository, performs the reverse action,
/// and marks the operation as undone.
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
    /// Undoes the operation with the given ID.
    async fn undo(&self, operation_id: OperationId) -> Result<OperationResult, UseCaseError> {
        // 1. Load the operation from the repository
        let mut op = self.operation_repo
            .find_by_id(&operation_id)
            .await
            .map_err(|e| UseCaseError::RepositoryError(e.to_string()))?
            .ok_or_else(|| UseCaseError::OperationNotFound(operation_id.clone()))?;

        // 2. Validate that the operation can be undone (only Completed → Undone)
        // OLD: if op.state != quicksort_domain::OperationState::Completed {
        // NEW: use pattern match for clarity and completeness
        if !matches!(op.state, OperationState::Completed { .. }) {
            return Err(UseCaseError::UndoNotPossible(
                "Only completed operations can be undone".to_string(),
            ));
        }

        // 3. Perform the undo according to the operation type
        // Each helper returns Ok(()) on success or a UseCaseError
        match op.operation_type {
            OperationType::Move => self.undo_move(&mut op).await?,
            OperationType::Copy => self.undo_copy(&mut op).await?,
            OperationType::Delete => self.undo_delete(&mut op).await?,
            OperationType::Rename => self.undo_rename(&mut op).await?,
        }

        // 4. Mark the operation as undone using the domain method
        // OLD: op.state = OperationState::Undone;
        //      op.updated_at = self.clock.now();
        // NEW: use the aggregate's own method to preserve invariants
        let now = self.clock.now();
        op.mark_undone(now).map_err(|e| UseCaseError::DomainError(e.to_string()))?;

        // 5. Persist the updated operation
        self.operation_repo
            .save(&op)
            .await
            .map_err(|e| UseCaseError::RepositoryError(e.to_string()))?;

        // 6. Return a result DTO
        Ok(OperationResult {
            operation_id: op.id.clone(),
            state: OperationState::Undone,
            processed_files: 1,
            bytes_moved: 0,
        })
    }
}

// ===== Private helper methods for each operation type =====

impl UndoOperationUseCase {
    /// Undo Move: move the file back from the target folder to the original location.
    async fn undo_move(&self, op: &mut Operation) -> Result<(), UseCaseError> {
        // Extract the source (original) path and the target folder path
        let source_path = op.source_paths.first()
            .ok_or_else(|| UseCaseError::UndoNotPossible("No source path found".to_string()))?;

        let target_folder = op.target_folder_path.as_ref()
            .ok_or_else(|| UseCaseError::UndoNotPossible("No target folder for Move".to_string()))?;

        // Determine the file name from the source path
        // OLD: .as_str().ok_or(...) – WindowsPath may not have as_str()
        // NEW: convert to a string via Display/ToString
        let file_name = source_path
            .file_name()
            .ok_or_else(|| UseCaseError::UndoNotPossible("Invalid source file name".to_string()))?;

        // Construct the full target path (target_folder + file_name)
        let target_path = target_folder.join(&file_name);

        // Verify that the file actually exists at the target location
        if !self.file_system.exists(&target_path).await
            .map_err(|e| UseCaseError::FileSystemError(e.to_string()))?
        {
            return Err(UseCaseError::UndoNotPossible(
                "File no longer exists at target location".to_string(),
            ));
        }

        // Perform the reverse move (rename back to source)
        self.file_system.rename_file(&target_path, source_path).await
            .map_err(|e| UseCaseError::FileSystemError(e.to_string()))?;

        Ok(())
    }

    /// Undo Copy: delete the copied file from the target folder.
    async fn undo_copy(&self, op: &mut Operation) -> Result<(), UseCaseError> {
        let source_path = op.source_paths.first()
            .ok_or_else(|| UseCaseError::UndoNotPossible("No source path found".to_string()))?;

        let target_folder = op.target_folder_path.as_ref()
            .ok_or_else(|| UseCaseError::UndoNotPossible("No target folder for Copy".to_string()))?;

        let file_name = source_path
            .file_name()
            .ok_or_else(|| UseCaseError::UndoNotPossible("Invalid source file name".to_string()))?;

        let target_path = target_folder.join(&file_name);

        // Only delete if the file still exists (it might have been removed already)
        if self.file_system.exists(&target_path).await
            .map_err(|e| UseCaseError::FileSystemError(e.to_string()))?
        {
            self.file_system.delete_file(&target_path).await
                .map_err(|e| UseCaseError::FileSystemError(e.to_string()))?;
        }

        Ok(())
    }

    /// Undo Delete: restore the deleted file.
    /// Currently unsupported – requires trash can integration.
    async fn undo_delete(&self, _op: &mut Operation) -> Result<(), UseCaseError> {
        // TODO: TASK-020 — Implement undo for Delete using trash can (IFileOperation with
        // FO_MOVE to recycle bin). For now, return a clear error.
        Err(UseCaseError::UndoNotPossible(
            "Undo of Delete operation requires trash can implementation".to_string(),
        ))
    }

    /// Undo Rename: restore the original file name.
    async fn undo_rename(&self, op: &mut Operation) -> Result<(), UseCaseError> {
        // For rename, source_paths[0] holds the old path,
        // target_paths[0] holds the new (current) path.
        let old_path = op.source_paths.first()
            .ok_or_else(|| UseCaseError::UndoNotPossible("No source path found".to_string()))?;

        let new_path = op.target_paths.as_ref()
            .and_then(|paths| paths.first())
            .ok_or_else(|| UseCaseError::UndoNotPossible("No target path for Rename".to_string()))?;

        // Verify that the renamed file still exists
        if !self.file_system.exists(new_path).await
            .map_err(|e| UseCaseError::FileSystemError(e.to_string()))?
        {
            return Err(UseCaseError::UndoNotPossible(
                "File with new name no longer exists".to_string(),
            ));
        }

        // Rename back to the original name
        self.file_system.rename_file(new_path, old_path).await
            .map_err(|e| {
                // Distinguish between file-system errors and other failures
                UseCaseError::FileSystemError(e.to_string())
            })?;

        Ok(())
    }
}