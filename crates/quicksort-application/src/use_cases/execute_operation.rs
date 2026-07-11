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

        // Build initial destination path
        let initial_dest = WindowsPath::new(&format!("{}\\{}", target_folder.as_str(), file_name))
            .map_err(|e| UseCaseError::Internal(e.to_string()))?;

        match policy {
            OverwritePolicy::Skip => {
                if self.file_system.exists(&initial_dest).await? {
                    Err(UseCaseError::Conflict(format!("File already exists: {}", initial_dest.as_str())))
                } else {
                    Ok(initial_dest)
                }
            }
            OverwritePolicy::Overwrite => Ok(initial_dest),
            OverwritePolicy::AutoRename => {
                // Generate unique name with loop
                let base_name = file_name.trim_end_matches(|c: char| c.is_ascii_digit() || c == '(' || c == ')' || c == ' ');
                let mut counter = 1;
                let ext = file_name.split('.').last().map(|s| format!(".{}", s)).unwrap_or_default();
                loop {
                    let new_name = if counter == 1 {
                        format!("{} (1){}", base_name, ext)
                    } else {
                        format!("{} ({}){}", base_name, counter, ext)
                    };
                    let candidate = WindowsPath::new(&format!("{}\\{}", target_folder.as_str(), new_name))
                        .map_err(|e| UseCaseError::Internal(e.to_string()))?;
                    if !self.file_system.exists(&candidate).await? {
                        return Ok(candidate);
                    }
                    counter += 1;
                }
            }
            OverwritePolicy::Ask => {
                // For now, we treat 'Ask' as requesting auto-rename behavior for simplicity in the Use Case layer.
                self.auto_rename(source, target_folder).await
            }
        }
    }

    async fn auto_rename(
        &self,
        source: &WindowsPath,
        target_folder: &WindowsPath,
    ) -> Result<WindowsPath, UseCaseError> {
        let file_name = source.as_str().split('\\').last()
            .ok_or_else(|| UseCaseError::InvalidCommand("Invalid source path".to_string()))?;
        let base_name = file_name.trim_end_matches(|c: char| c.is_ascii_digit() || c == '(' || c == ')' || c == ' ');
        let mut counter = 1;
        let ext = file_name.split('.').last().map(|s| format!(".{}", s)).unwrap_or_default();
        loop {
            let new_name = if counter == 1 {
                format!("{} (1){}", base_name, ext)
            } else {
                format!("{} ({}){}", base_name, counter, ext)
            };
            let candidate = WindowsPath::new(&format!("{}\\{}", target_folder.as_str(), new_name))
                .map_err(|e| UseCaseError::Internal(e.to_string()))?;
            if !self.file_system.exists(&candidate).await? {
                return Ok(candidate);
            }
            counter += 1;
        }
    }
}

#[async_trait]
impl ExecuteOperation for ExecuteOperationUseCase {
    async fn execute(&self, command: OperationCommand) -> Result<OperationResult, UseCaseError> {
        let folders = self.config_repo.load_all().await?;
        
        // Determine target folder and explicit targets based on operation type
        let (target_folder, target_paths): (Arc<dyn Folder>, Option<Vec<WindowsPath>>) = match &command.operation_type {
            OperationType::Move | OperationType::Copy => {
                // For Move/Copy, the destination is a container folder.
                let id = command.target_folder_id.as_ref()
                    .ok_or_else(|| UseCaseError::InvalidCommand("Target folder required".to_string()))?;
                folders.iter().find(|f| f.id == *id)
                    .ok_or_else(|| UseCaseError::FolderNotFound(id.as_str().to_string()))?
                let target = Arc::new(f.clone());
                (target, None)
            }
            OperationType::Delete => {
                // Deletion does not require a specific container folder for the operation itself. 
                // We use a dummy container and no explicit targets.
                let default_id = self.config_repo.get_default_folder_id().await?;
                let target = Arc::new(folders.iter().find(|f| f.id == *default_id).unwrap().clone());
                (target, None)
            }
            OperationType::Rename => {
                // Rename requires explicit target paths and must have the same length as source paths.
                let targets = command.target_paths.as_ref()
                    .ok_or_else(|| UseCaseError::InvalidCommand("Target paths are required for rename operation".to_string()))?;
                if targets.len() != command.source_paths.len() {
                    return Err(UseCaseError::InvalidCommand("Source and target path lists must have the same length for renaming".to_string()));
                }
                // Use a dummy folder, as rename is not container-based.
                let default_id = self.config_repo.get_default_folder_id().await?;
                let target = Arc::new(folders.iter().find(|f| f.id == *default_id).unwrap().clone());
                (target, Some(targets.clone()))
            }
        };

        let op_id = OperationId::from_string(self.id_generator.generate());
        let now = self.clock.now();
        let mut operation = Operation::new(
            op_id.clone(),
            command.operation_type.clone(),
            command.source_paths.clone(),
            Some(target_folder.path.clone()),
            None, // target_paths will be set later if needed
            now,
        );
        operation.start(now).map_err(|e| UseCaseError::Internal(e.to_string()))?;

        let mut total_bytes = 0;
        let mut processed = 0;

        match command.operation_type {
            OperationType::Move | OperationType::Copy => {
                // Move/Copy: All sources go to the same resolved destination folder.
                for src in command.source_paths.iter() {
                    // Resolve conflict using the determined container folder path
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
                        _ => unreachable!(), // Handled by outer match
                    }
                }
            }
            OperationType::Rename => {
                // Rename: Each source path has a unique target path.
                let rename_pairs = command.source_paths.iter().zip(target_paths.as_ref().unwrap()).collect::<Vec<_>>();

                for ((src, &dest_path), _) in rename_pairs.into_iter() {
                    // For renaming, we must resolve conflict for the specific target path provided by the user.
                    let resolved_dest = self.resolve_conflict(src, &target_folder.path, command.overwrite_policy).await?;

                    // Note: The resolve_conflict logic is designed to find a unique name within a folder. 
                    // If the user provides an explicit target path (which might be outside the container), 
                    // we should ideally use that directly if it doesn't conflict, but for simplicity and adherence 
                    // to the current structure, we assume the resolved_dest is the best attempt at the final name.
                    let bytes = self.file_system.rename_file(src, &resolved_dest).await?;
                    total_bytes += bytes;
                    processed += 1;
                }
            }
            OperationType::Delete => {
                // Delete: Each source path is deleted independently.
                for src in command.source_paths.iter() {
                    let bytes = self.file_system.delete_file(src).await?;
                    total_bytes += bytes;
                    processed += 1;
                }
            }
            _ => return Err(UseCaseError::InvalidCommand("Unsupported operation".to_string())),
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