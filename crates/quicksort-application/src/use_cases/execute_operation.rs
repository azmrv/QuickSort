//! ExecuteOperationUseCase - orchestrates file operations (move, copy, delete, rename).
//!
//! # Design Decisions
//! - Resolves destination paths with conflict handling according to the chosen policy.
//! - Relies on `FileSystem` port to retrieve file sizes after operations.
//! - Delegates state transitions to the `Operation` aggregate (no direct state manipulation).

use async_trait::async_trait;
use std::sync::Arc;
use quicksort_domain::{Operation, OperationId, OperationType, OperationState, WindowsPath};
use crate::dtos::{OperationCommand, OperationResult, OverwritePolicy};
use crate::errors::UseCaseError;
use crate::ports::inbound::ExecuteOperation;
use crate::ports::outbound::{
    ConfigurationRepository, OperationRepository, FileSystem,
    IdGenerator, Clock,
};

/// Use case responsible for executing Move, Copy, Delete, and Rename operations.
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

    /// Determine the final destination path for a single source file,
    /// applying the given overwrite policy.
    async fn resolve_destination(
        &self,
        source: &WindowsPath,
        target_folder: &WindowsPath,
        policy: OverwritePolicy,
    ) -> Result<WindowsPath, UseCaseError> {
        // Extract the file name from the source path
        // OLD: source.as_str().ok_or(...).split('\\').last()...
        // NEW: use Path methods – safer and more idiomatic
        let file_name = source
            .file_name()
            .ok_or_else(|| UseCaseError::InvalidCommand("Invalid source file name".to_string()))?;

        // Construct the initial destination path
        let initial_dest = target_folder.join(&file_name);

        match policy {
            // If the file already exists and policy is Skip, return a conflict error.
            OverwritePolicy::Skip => {
                if self.file_system.exists(&initial_dest).await
                    .map_err(|e| UseCaseError::FileSystemError(e.to_string()))?
                {
                    Err(UseCaseError::Conflict(format!(
                        "File already exists: {}",
                        // OLD: initial_dest.as_str().ok_or(...)
                        // NEW: use Display / to_string_lossy for logging
                        initial_dest.to_string_lossy()
                    )))
                } else {
                    Ok(initial_dest)
                }
            }

            // Overwrite the existing file unconditionally.
            OverwritePolicy::Overwrite => Ok(initial_dest),

            // Append a numeric suffix until an unused name is found.
            OverwritePolicy::AutoRename => {
                let base_name = source
                    .file_stem()
                    .unwrap_or_else(|| file_name.to_str().unwrap_or("unnamed"));
                let ext = source.extension();

                let mut counter = 1u32;
                loop {
                    let candidate = if let Some(ext) = ext {
                        target_folder.join(format!("{} ({}).{}", base_name, counter, ext.to_string_lossy()))
                    } else {
                        target_folder.join(format!("{} ({})", base_name, counter))
                    };

                    if !self.file_system.exists(&candidate).await
                        .map_err(|e| UseCaseError::FileSystemError(e.to_string()))?
                    {
                        return Ok(candidate);
                    }
                    counter = counter.saturating_add(1);
                }
            }

            // Ask policy should be handled at the adapter layer (GUI/CLI).
            // If it reaches the Use Case from a non-interactive context, fall back to AutoRename.
            OverwritePolicy::Ask => {
                self.resolve_destination(source, target_folder, OverwritePolicy::AutoRename).await
            }
        }
    }
}

#[async_trait]
impl ExecuteOperation for ExecuteOperationUseCase {
    /// Execute a file operation command and return the result.
    async fn execute(&self, command: OperationCommand) -> Result<OperationResult, UseCaseError> {
        // 1. Load all configured folders (needed to resolve target_folder_id -> path)
        let folders = self.config_repo.load_all().await
            .map_err(|e| UseCaseError::RepositoryError(e.to_string()))?;

        // 2. Create the operation aggregate
        // OLD: let now = self.clock.now();
        //      Operation::new(op_id, ..., now) and operation.start(now), operation.complete(..., now)
        // NEW: start() and complete() no longer take a time parameter
        let op_id = OperationId::new();

        // Determine the target folder and explicit target paths based on operation type
        let (target_folder_path, target_paths) = match &command.operation_type {
            OperationType::Move | OperationType::Copy => {
                let id = command
                    .target_folder_id
                    .as_ref()
                    .ok_or_else(|| UseCaseError::InvalidCommand(
                        "Target folder ID is required for Move/Copy".to_string()
                    ))?;
                let folder = folders
                    .iter()
                    .find(|f| f.id == *id)
                    .ok_or_else(|| UseCaseError::FolderNotFound(id.clone()))?;
                (Some(folder.path.clone()), None)
            }
            OperationType::Delete => (None, None),
            OperationType::Rename => {
                let paths = command
                    .target_paths
                    .as_ref()
                    .ok_or_else(|| UseCaseError::InvalidCommand(
                        "Target paths are required for Rename".to_string()
                    ))?;
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
            // OLD: now,
            // NEW: Operation::new still requires a timestamp for creation;
            // keep the clock call for that purpose, but do not pass it to start/complete
            self.clock.now(),
        );

        // 3. Start the operation (Pending → Executing)
        // OLD: operation.start(now).map_err(|_| ...)
        // NEW: start() no longer takes a time argument; it records Utc::now() internally
        operation.start()
            .map_err(|e| UseCaseError::Domain(e.to_string()))?;

        let mut total_bytes: u64 = 0;
        let mut processed: u32 = 0;

        // 4. Perform the actual file system operations
        match &command.operation_type {
            OperationType::Move => {
                let target_folder = folders
                    .iter()
                    .find(|f| f.id == command.target_folder_id.as_ref().unwrap())
                    .ok_or_else(|| UseCaseError::FolderNotFound("Target folder".to_string()))?;

                for src in &command.source_paths {
                    let dest = self
                        .resolve_destination(src, &target_folder.path, command.overwrite_policy)
                        .await?;
                    // OLD: let _: u64 = self.file_system.move_file(src, &dest).await?;
                    //      total_bytes += &0u64;  <-- dead assignment
                    // NEW: store the returned file size
                    let bytes = self.file_system.move_file(src, &dest).await
                        .map_err(|e| UseCaseError::FileSystemError(e.to_string()))?;
                    total_bytes += bytes;
                    processed += 1;
                }
            }
            OperationType::Copy => {
                let target_folder = folders
                    .iter()
                    .find(|f| f.id == command.target_folder_id.as_ref().unwrap())
                    .ok_or_else(|| UseCaseError::FolderNotFound("Target folder".to_string()))?;

                for src in &command.source_paths {
                    let dest = self
                        .resolve_destination(src, &target_folder.path, command.overwrite_policy)
                        .await?;
                    let bytes = self.file_system.copy_file(src, &dest).await
                        .map_err(|e| UseCaseError::FileSystemError(e.to_string()))?;
                    total_bytes += bytes;
                    processed += 1;
                }
            }
            OperationType::Delete => {
                for src in &command.source_paths {
                    let bytes = self.file_system.delete_file(src).await
                        .map_err(|e| UseCaseError::FileSystemError(e.to_string()))?;
                    total_bytes += bytes;
                    processed += 1;
                }
            }
            OperationType::Rename => {
                let target_paths = command
                    .target_paths
                    .as_ref()
                    .ok_or_else(|| UseCaseError::InvalidCommand("Target paths missing".to_string()))?;

                for (src, dest) in command.source_paths.iter().zip(target_paths.iter()) {
                    let bytes = self.file_system.rename_file(src, dest).await
                        .map_err(|e| UseCaseError::FileSystemError(e.to_string()))?;
                    total_bytes += bytes;
                    processed += 1;
                }
            }
        }

        // 5. Mark the operation as completed
        // OLD: operation.complete(processed, total_bytes, now).map_err(|_| ...)
        // NEW: complete() no longer takes a time argument
        operation.complete(processed, total_bytes)
            .map_err(|e| UseCaseError::Domain(e.to_string()))?;

        // 6. Save the operation for history and potential undo
        self.operation_repo.save(&operation).await
            .map_err(|e| UseCaseError::RepositoryError(e.to_string()))?;

        Ok(OperationResult {
            operation_id: op_id,
            state: operation.state,
            processed_files: processed,
            bytes_moved: total_bytes,
        })
    }
}