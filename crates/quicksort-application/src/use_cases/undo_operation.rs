use crate::dtos::OperationResult;
use crate::errors::UseCaseError;
use crate::ports::inbound::UndoOperation;
use crate::ports::outbound::{FileSystem, OperationRepository};
use async_trait::async_trait;
use quicksort_domain::{Operation, OperationId, OperationState, OperationType};

pub struct UndoOperationUseCase {
    operation_repo: Box<dyn OperationRepository>,
    file_system: Box<dyn FileSystem>,
}

impl UndoOperationUseCase {
    pub fn new(
        operation_repo: Box<dyn OperationRepository>,
        file_system: Box<dyn FileSystem>,
    ) -> Self {
        Self {
            operation_repo,
            file_system,
        }
    }
}

#[async_trait]
impl UndoOperation for UndoOperationUseCase {
    async fn undo(&self, operation_id: OperationId) -> Result<OperationResult, UseCaseError> {
        let mut op = self
            .operation_repo
            .find_by_id(&operation_id)
            .await
            .map_err(|e| UseCaseError::RepositoryError(e.to_string()))?
            .ok_or_else(|| UseCaseError::OperationNotFound(operation_id.to_string()))?;

        if !matches!(op.state, OperationState::Completed { .. }) {
            return Err(UseCaseError::UndoNotPossible(
                "Only completed operations can be undone".to_string(),
            ));
        }

        match op.operation_type {
            OperationType::Move => self.undo_move(&mut op).await?,
            OperationType::Copy => self.undo_copy(&mut op).await?,
            OperationType::Delete => self.undo_delete(&mut op).await?,
            OperationType::Rename => self.undo_rename(&mut op).await?,
        }

        op.mark_undone()
            .map_err(|e| UseCaseError::Domain(e.to_string()))?;

        self.operation_repo
            .save(&op)
            .await
            .map_err(|e| UseCaseError::RepositoryError(e.to_string()))?;

        Ok(OperationResult {
            operation_id: op.id,
            state: OperationState::Undone,
            processed_files: op.source_paths.len() as u32,
            bytes_moved: 0,
        })
    }
}

impl UndoOperationUseCase {
    async fn undo_move(&self, op: &mut Operation) -> Result<(), UseCaseError> {
        let target_folder = op.target_folder_path.as_ref().ok_or_else(|| {
            UseCaseError::UndoNotPossible("No target folder for Move".to_string())
        })?;

        for source_path in &op.source_paths {
            let file_name = source_path.file_name().ok_or_else(|| {
                UseCaseError::UndoNotPossible("Invalid source file name".to_string())
            })?;

            let target_path = target_folder.join(file_name);

            if !self
                .file_system
                .exists(&target_path)
                .await
                .map_err(|e| UseCaseError::FileSystemError(e.to_string()))?
            {
                return Err(UseCaseError::UndoNotPossible(format!(
                    "File no longer exists at target location: {}",
                    target_path.display()
                )));
            }

            self.file_system
                .rename_file(&target_path, source_path)
                .await
                .map_err(|e| UseCaseError::FileSystemError(e.to_string()))?;
        }

        Ok(())
    }

    async fn undo_copy(&self, op: &mut Operation) -> Result<(), UseCaseError> {
        let target_folder = op.target_folder_path.as_ref().ok_or_else(|| {
            UseCaseError::UndoNotPossible("No target folder for Copy".to_string())
        })?;

        for source_path in &op.source_paths {
            let file_name = source_path.file_name().ok_or_else(|| {
                UseCaseError::UndoNotPossible("Invalid source file name".to_string())
            })?;

            let target_path = target_folder.join(file_name);

            if self
                .file_system
                .exists(&target_path)
                .await
                .map_err(|e| UseCaseError::FileSystemError(e.to_string()))?
            {
                self.file_system
                    .delete_file(&target_path)
                    .await
                    .map_err(|e| UseCaseError::FileSystemError(e.to_string()))?;
            }
        }

        Ok(())
    }

    async fn undo_delete(&self, _op: &mut Operation) -> Result<(), UseCaseError> {
        Err(UseCaseError::UndoNotPossible(
            "Undo of Delete operation requires trash can implementation".to_string(),
        ))
    }

    async fn undo_rename(&self, op: &mut Operation) -> Result<(), UseCaseError> {
        let target_paths = op.target_paths.as_ref().ok_or_else(|| {
            UseCaseError::UndoNotPossible("No target paths for Rename".to_string())
        })?;

        for (old_path, new_path) in op.source_paths.iter().zip(target_paths.iter()) {
            if !self
                .file_system
                .exists(new_path)
                .await
                .map_err(|e| UseCaseError::FileSystemError(e.to_string()))?
            {
                return Err(UseCaseError::UndoNotPossible(format!(
                    "File with new name no longer exists: {}",
                    new_path.display()
                )));
            }

            self.file_system
                .rename_file(new_path, old_path)
                .await
                .map_err(|e| UseCaseError::FileSystemError(e.to_string()))?;
        }

        Ok(())
    }
}
