//! ExecuteOperationUseCase - orchestrates file operations (move, copy, etc.)

use async_trait::async_trait;
use std::sync::Arc;
use quicksort_domain::{Operation, OperationId, OperationType, OperationState, WindowsPath, DomainEvent};
use crate::dtos::{OperationCommand, OperationResult, OverwritePolicy};
use crate::errors::UseCaseError;
use crate::ports::outbound::{
    ConfigurationRepository, OperationRepository, FileSystem,
    IdGenerator, Clock, ConflictResolver,
};
use crate::ports::inbound::ExecuteOperation;

pub struct ExecuteOperationUseCase {
    config_repo: Arc<dyn ConfigurationRepository>,
    operation_repo: Arc<dyn OperationRepository>,
    file_system: Arc<dyn FileSystem>,
    id_generator: Arc<dyn IdGenerator>,
    clock: Arc<dyn Clock>,
    conflict_resolver: Arc<dyn ConflictResolver>,
}

impl ExecuteOperationUseCase {
    pub fn new(
        config_repo: Arc<dyn ConfigurationRepository>,
        operation_repo: Arc<dyn OperationRepository>,
        file_system: Arc<dyn FileSystem>,
        id_generator: Arc<dyn IdGenerator>,
        clock: Arc<dyn Clock>,
        conflict_resolver: Arc<dyn ConflictResolver>,
    ) -> Self {
        Self {
            config_repo,
            operation_repo,
            file_system,
            id_generator,
            clock,
            conflict_resolver,
        }
    }

    /// Resolves conflict: returns the actual destination path after applying policy.
    async fn resolve_conflict(
        &self,
        source: &WindowsPath,
        target_folder: &WindowsPath,
        policy: OverwritePolicy,
    ) -> Result<WindowsPath, UseCaseError> {
        let file_name = source.as_str().split('\\').last()
            .ok_or_else(|| UseCaseError::InvalidCommand("Invalid source path".to_string()))?;
        let mut dest_path = WindowsPath::new(&format!("{}\\{}", target_folder.as_str(), file_name))
            .map_err(|e| UseCaseError::Internal(e.to_string()))?;

        let exists = self.file_system.exists(&dest_path).await?;
        if !exists {
            return Ok(dest_path);
        }

        match policy {
            OverwritePolicy::Skip => {
                Err(UseCaseError::Conflict(format!("File already exists: {}", dest_path.as_str())))
            }
            OverwritePolicy::Overwrite => Ok(dest_path),
            OverwritePolicy::AutoRename => {
                let base_name = file_name.trim_end_matches(|c: char| c.is_ascii_digit() || c == '(' || c == ')' || c == ' ');
                let mut counter = 1;
                loop {
                    let new_name = format!("{} ({})", base_name, counter);
                    let candidate = WindowsPath::new(&format!("{}\\{}", target_folder.as_str(), new_name))
                        .map_err(|e| UseCaseError::Internal(e.to_string()))?;
                    if !self.file_system.exists(&candidate).await? {
                        return Ok(candidate);
                    }
                    counter += 1;
                }
            }
            OverwritePolicy::Ask => {
                // In production, this would trigger a user prompt via adapter.
                // For now, fallback to AutoRename.
                self.resolve_conflict(source, target_folder, OverwritePolicy::AutoRename).await
            }
        }
    }
}

#[async_trait]
impl ExecuteOperation for ExecuteOperationUseCase {
    async fn execute(&self, command: OperationCommand) -> Result<OperationResult, UseCaseError> {
        let folders = self.config_repo.load_all().await?;
        let target_folder = match &command.operation_type {
            OperationType::Move | OperationType::Copy => {
                let id = command.target_folder_id.as_ref()
                    .ok_or_else(|| UseCaseError::InvalidCommand("Target folder required".to_string()))?;
                folders.iter().find(|f| f.id == *id)
                    .ok_or_else(|| UseCaseError::FolderNotFound(id.as_str().to_string()))?
            }
            OperationType::Delete => {
                return Err(UseCaseError::InvalidCommand("Delete not implemented".to_string()));
            }
            OperationType::Rename => {
                return Err(UseCaseError::InvalidCommand("Rename not implemented".to_string()));
            }
        };

        let op_id = OperationId::from_string(self.id_generator.generate());
        let now = self.clock.now();
        let mut operation = Operation::new(
            op_id.clone(),
            command.operation_type.clone(),
            command.source_paths.clone(),
            Some(target_folder.path.clone()),
            now,
        );
        operation.start(now).map_err(|e| UseCaseError::Internal(e.to_string()))?;

        let mut total_bytes = 0;
        let mut processed = 0;

        for src in &command.source_paths {
            let dest = self.resolve_conflict(src, &target_folder.path, command.overwrite_policy).await?;
            match command.operation_type {
                OperationType::Move => {
                    let bytes = self.file_system.move_file(src, &dest).await?;
                    total_bytes += bytes;
                    processed += 1;
                }
                OperationType::Copy => {
                    let bytes = self.file_system.copy_file(src, &dest).await?;
                    total_bytes += bytes;
                    processed += 1;
                }
                _ => return Err(UseCaseError::InvalidCommand("Unsupported operation".to_string())),
            }
        }

        operation.complete(processed, total_bytes, now)
            .map_err(|e| UseCaseError::Internal(e.to_string()))?;
        self.operation_repo.save(&operation).await?;
        let _events = operation.pull_events();

        Ok(OperationResult {
            operation_id: op_id,
            state: operation.state,
            processed_files: processed,
            bytes_moved: total_bytes,
        })
    }
}