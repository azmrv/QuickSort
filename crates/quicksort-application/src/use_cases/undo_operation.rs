// UndoOperationUseCase - reverts a completed operation.

use async_trait::async_trait;
use std::sync::Arc;
use quicksort_domain::{OperationId, OperationState, OperationType, WindowsPath};
use crate::dtos::OperationResult;
use crate::errors::UseCaseError;
use crate::ports::outbound::{OperationRepository, FileSystem, Clock};
use crate::ports::inbound::UndoOperation;

/// Реализация Use Case отката операций.
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
    /// Выполняет откат операции с указанным ID.
    async fn undo(&self, operation_id: OperationId) -> Result<OperationResult, UseCaseError> {
        // 1. Load operation
        let mut op = self.operation_repo
            .find_by_id(&operation_id)
            .await
            .map_err(|e| UseCaseError::RepositoryError(e.to_string()))?
            .ok_or_else(|| UseCaseError::OperationNotFound(operation_id.clone()))?;

        // 2. Check if operation is completed and not already undone
        if op.state != OperationState::Completed {
            return Err(UseCaseError::UndoNotPossible(
                "Only completed operations can be undone".to_string(),
            ));
        }

        // 3. Perform undo based on operation type
        let result = match op.operation_type {
            OperationType::Move => self.undo_move(&mut op).await,
            OperationType::Copy => self.undo_copy(&mut op).await,
            OperationType::Delete => self.undo_delete(&mut op).await,
            OperationType::Rename => self.undo_rename(&mut op).await,
        };

        // 4. If successful, update operation state to Undone
        match result {
            Ok(_) => {
                op.state = OperationState::Undone;
                op.updated_at = self.clock.now();
                
                self.operation_repo
                    .save(&op)
                    .await
                    .map_err(|e| UseCaseError::RepositoryError(e.to_string()))?;

                Ok(OperationResult {
                    operation_id: op.id.clone(),
                    state: OperationState::Undone,
                    processed_files: 1, // По умолчанию для одной операции
                    bytes_moved: 0,
                })
            }
            Err(e) => Err(e),
        }
    }
}

// ===== Helper methods =====

impl UndoOperationUseCase {
    /// Откат Move операции: перемещает файл обратно из цели в исходную позицию.
    async fn undo_move(&self, op: &mut quicksort_domain::Operation) -> Result<(), UseCaseError> {
        let source_path = op.source_paths.first()
            .ok_or_else(|| UseCaseError::UndoNotPossible("No source path found".to_string()))?;
        let target_folder = op.target_folder_path.as_ref()
            .ok_or_else(|| UseCaseError::UndoNotPossible("No target folder for Move".to_string()))?;

        // Determine destination file name (use the same file name as source)
        let file_name = source_path.as_str().split('\\').last()
            .ok_or_else(|| UseCaseError::UndoNotPossible("Invalid source path".to_string()))?;
        let target_path = WindowsPath::new(&format!("{}\\{}", target_folder.as_str(), file_name))
            .map_err(|_| UseCaseError::UndoNotPossible("Invalid target path".to_string()))?;

        // Check if file exists at target
        if !self.file_system.exists(&target_path).await
            .map_err(|e| UseCaseError::FileSystemError(e.to_string()))?
        {
            return Err(UseCaseError::UndoNotPossible(
                "File no longer exists at target location".to_string(),
            ));
        }

        // Move back to source
        self.file_system.rename_file(&target_path, source_path).await
            .map_err(|e| UseCaseError::FileSystemError(e.to_string()))?;

        Ok(())
    }

    /// Откат Copy операции: удаляет копию файла из цели.
    async fn undo_copy(&self, op: &mut quicksort_domain::Operation) -> Result<(), UseCaseError> {
        let target_folder = op.target_folder_path.as_ref()
            .ok_or_else(|| UseCaseError::UndoNotPossible("No target folder for Copy".to_string()))?;
        let source_path = op.source_paths.first()
            .ok_or_else(|| UseCaseError::UndoNotPossible("No source path found".to_string()))?;

        // Determine copied file name
        let file_name = source_path.as_str().split('\\').last()
            .ok_or_else(|| UseCaseError::UndoNotPossible("Invalid source path".to_string()))?;
        let target_path = WindowsPath::new(&format!("{}\\{}", target_folder.as_str(), file_name))
            .map_err(|_| UseCaseError::UndoNotPossible("Invalid target path".to_string()))?;

        // Remove the copied file if it exists
        if self.file_system.exists(&target_path).await
            .map_err(|e| UseCaseError::FileSystemError(e.to_string()))?
        {
            self.file_system.delete_file(&target_path).await
                .map_err(|e| UseCaseError::FileSystemError(e.to_string()))?;
        }

        Ok(())
    }

    /// Откат Delete операции: восстанавливает удалённый файл из корзины.
    async fn undo_delete(&self, op: &mut quicksort_domain::Operation) -> Result<(), UseCaseError> {
        // TODO: Реализация с поддержкой корзины (Trash can pattern)
        // Пока возвращаем ошибку - не поддерживается без механизма корзины
        Err(UseCaseError::UndoNotPossible(
            "Undo of Delete operation requires trash can implementation".to_string(),
        ))
    }

    /// Откат Rename операции: возвращает исходное имя файла.
    async fn undo_rename(&self, op: &mut quicksort_domain::Operation) -> Result<(), UseCaseError> {
        // Для отката rename нам нужны old_name и new_name поля в Operation
        let old_name = &op.old_name;
        let file_path = &op.source_paths.first()
            .ok_or_else(|| UseCaseError::UndoNotPossible("No source path found".to_string()))?;

        // Construct new path with old name
        let path_str = file_path.as_str();
        let parent = path_str.rsplit_once('\\')
            .map(|(p, _)| p)
            .unwrap_or("");
        
        let new_path_str = if parent.is_empty() {
            old_name.to_string()
        } else {
            format!("{}\\{}", parent, old_name)
        };

        let new_path = WindowsPath::new(new_path_str)
            .map_err(|_| UseCaseError::UndoNotPossible("Invalid path construction".to_string()))?;

        // Rename back to original name
        self.file_system.rename_file(file_path, &new_path).await
            .map_err(|e| UseCaseError::FileSystemError(e.to_string()))?;

        Ok(())
    }
}