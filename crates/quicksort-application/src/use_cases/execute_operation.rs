//! ExecuteOperationUseCase - orchestrates file operations (move, copy, delete, rename).

use async_trait::async_trait;
use std::sync::Arc;
use quicksort_domain::{Operation, OperationId, OperationType, WindowsPath};
use crate::dtos::{OperationCommand, OperationResult, OverwritePolicy};
use crate::errors::UseCaseError;
use crate::ports::inbound::ExecuteOperation;
use crate::ports::outbound::{
    ConfigurationRepository, OperationRepository, FileSystem,
    IdGenerator, Clock,
};

pub struct ExecuteOperationUseCase {
    config_repo: Arc<dyn ConfigurationRepository>,
    operation_repo: Arc<dyn OperationRepository>,
    file_system: Arc<dyn FileSystem>,
    id_generator: Arc<dyn IdGenerator>,
    clock: Arc<dyn Clock>,
}

impl ExecuteOperationUseCase {
    pub fn new(
        config_repo: Arc<dyn ConfigurationRepository>,
        operation_repo: Arc<dyn OperationRepository>,
        file_system: Arc<dyn FileSystem>,
        id_generator: Arc<dyn IdGenerator>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            config_repo,
            operation_repo,
            file_system,
            id_generator,
            clock,
        }
    }

    /// Resolves conflict and returns the final destination path.
    async fn resolve_destination(
        &self,
        source: &WindowsPath,
        target_folder: &WindowsPath,
        policy: OverwritePolicy,
    ) -> Result<WindowsPath, UseCaseError> {
        let file_name = source
            .as_str()
            .ok_or_else(|| UseCaseError::InvalidCommand("Invalid source path".to_string()))?
            .split('\\')
            .last()
            .ok_or_else(|| UseCaseError::InvalidCommand("Invalid file name".to_string()))?;

        let folder_path = target_folder
            .as_str()
            .ok_or_else(|| UseCaseError::Internal("Invalid target folder path".to_string()))?;

        let initial_dest = WindowsPath::new(&format!("{}\\{}", folder_path, file_name))
            .map_err(|e| UseCaseError::Internal(e.to_string()))?;

        match policy {
            OverwritePolicy::Skip => {
                if self.file_system.exists(&initial_dest).await? {
                    let dest = initial_dest
                        .as_str()
                        .ok_or_else(|| UseCaseError::Internal("Invalid destination path".to_string()))?;
                    Err(UseCaseError::Conflict(format!("File already exists: {}", dest)))
                } else {
                    Ok(initial_dest)
                }
            }
            OverwritePolicy::Overwrite => Ok(initial_dest),
            OverwritePolicy::AutoRename => {
                let mut counter = 1;
                let base_name = file_name;
                let ext = source
                    .extension()
                    .map(|e| format!(".{}", e))
                    .unwrap_or_default();

                loop {
                    let new_name = if counter == 1 {
                        format!("{} (1){}", base_name, ext)
                    } else {
                        format!("{} ({}){}", base_name, counter, ext)
                    };
                    let candidate = WindowsPath::new(&format!("{}\\{}", folder_path, new_name))
                        .map_err(|e| UseCaseError::Internal(e.to_string()))?;
                    if !self.file_system.exists(&candidate).await? {
                        return Ok(candidate);
                    }
                    counter += 1;
                }
            }
            // Ask policy is handled at the adapter layer (GUI/CLI).
            // Fallback to AutoRename if Ask is passed from a non-interactive context.
            OverwritePolicy::Ask => {
                // For non-interactive contexts, treat Ask as AutoRename.
                self.resolve_destination(source, target_folder, OverwritePolicy::AutoRename).await
            }
        }
    }
}

#[async_trait]
impl ExecuteOperation for ExecuteOperationUseCase {
    async fn execute(&self, command: OperationCommand) -> Result<OperationResult, UseCaseError> {
        let folders = self.config_repo.load_all().await?;
        let now = self.clock.now();
        let op_id = OperationId::new();

        // Determine target folder path and explicit target paths based on operation type
        let (target_folder_path, target_paths) = match &command.operation_type {
            OperationType::Move | OperationType::Copy => {
                let id = command
                    .target_folder_id
                    .as_ref()
                    .ok_or_else(|| UseCaseError::InvalidCommand("Target folder required for Move/Copy".to_string()))?;
                let folder = folders
                    .iter()
                    .find(|f| &f.id == id)
                    .ok_or_else(|| UseCaseError::FolderNotFound(id.to_string()))?;
                (Some(folder.path.clone()), None)
            }
            OperationType::Delete => (None, None),
            OperationType::Rename => {
                let paths = command
                    .target_paths
                    .as_ref()
                    .ok_or_else(|| UseCaseError::InvalidCommand("Target paths required for Rename".to_string()))?;
                if paths.len() != command.source_paths.len() {
                    return Err(UseCaseError::InvalidCommand(
                        "Source and target path counts must match for Rename".to_string(),
                    ));
                }
                (None, Some(paths.clone()))
            }
        };

        let mut operation = Operation::new(
            op_id.clone(),
            command.operation_type.clone(),
            command.source_paths.clone(),
            target_folder_path,
            target_paths,
            now,
        );

        operation
            .start(now)
            .map_err(|_| UseCaseError::InvalidState("Operation state transition failed".to_string()))?;

        let mut total_bytes = 0u64;
        let mut processed = 0;

        match &command.operation_type {
            OperationType::Move => {
                let target_folder = folders
                    .iter()
                    .find(|f| &f.id == command.target_folder_id.as_ref().unwrap())
                    .ok_or_else(|| UseCaseError::FolderNotFound("Target folder".to_string()))?;

                for src in &command.source_paths {
                    let dest = self
                        .resolve_destination(src, &target_folder.path, command.overwrite_policy)
                        .await?;
                    let _: u64 = self.file_system.move_file(src, &dest).await?;
                    total_bytes += &0u64;
                    processed += 1;
                }
            }
            OperationType::Copy => {
                let target_folder = folders
                    .iter()
                    .find(|f| &f.id == command.target_folder_id.as_ref().unwrap())
                    .ok_or_else(|| UseCaseError::FolderNotFound("Target folder".to_string()))?;

                for src in &command.source_paths {
                    let dest = self
                        .resolve_destination(src, &target_folder.path, command.overwrite_policy)
                        .await?;
                    let _: u64 = self.file_system.copy_file(src, &dest).await?;
                    total_bytes += &0u64;
                    processed += 1;
                }
            }
            OperationType::Delete => {
                for src in &command.source_paths {
                    let _: u64 = self.file_system.delete_file(src).await?;
                    total_bytes += &0u64;
                    processed += 1;
                }
            }
            OperationType::Rename => {
                let target_paths = command
                    .target_paths
                    .as_ref()
                    .ok_or_else(|| UseCaseError::InvalidCommand("Target paths missing".to_string()))?;

                for (src, dest) in command.source_paths.iter().zip(target_paths.iter()) {
                    let _: u64 = self.file_system.rename_file(src, dest).await?;
                    total_bytes += &0u64;
                    processed += 1;
                }
            }
        }

        operation
            .complete(processed, total_bytes, now)
            .map_err(|_| UseCaseError::InvalidState("Operation state transition failed".to_string()))?;

        self.operation_repo.save(&operation).await?;

        Ok(OperationResult {
            operation_id: op_id,
            state: operation.state,
            processed_files: processed,
            bytes_moved: total_bytes,
        })
    }
}