use crate::dtos::{OperationCommand, OperationResult, OverwritePolicy};
use crate::errors::UseCaseError;
use crate::ports::inbound::ExecuteOperation;
use crate::ports::outbound::{
    Clock, ConfigurationRepository, DuplicateDetectionPort, FileSystem, IdGenerator,
    OperationRepository, ProgressInfo, ProgressReporter,
};
use async_trait::async_trait;
use quicksort_domain::{
    AbsolutePath, DuplicateCheckMode, Operation, OperationState, OperationType,
};

pub struct ExecuteOperationUseCase {
    operation_repository: Box<dyn OperationRepository>,
    configuration_repository: Box<dyn ConfigurationRepository>,
    file_system: Box<dyn FileSystem>,
    id_generator: Box<dyn IdGenerator>,
    clock: Box<dyn Clock>,
    duplicate_detector: Box<dyn DuplicateDetectionPort>,
    progress_reporter: Option<Box<dyn ProgressReporter>>,
}

impl ExecuteOperationUseCase {
    pub fn new(
        operation_repository: Box<dyn OperationRepository>,
        configuration_repository: Box<dyn ConfigurationRepository>,
        file_system: Box<dyn FileSystem>,
        id_generator: Box<dyn IdGenerator>,
        clock: Box<dyn Clock>,
        duplicate_detector: Box<dyn DuplicateDetectionPort>,
    ) -> Self {
        Self {
            operation_repository,
            configuration_repository,
            file_system,
            id_generator,
            clock,
            duplicate_detector,
            progress_reporter: None,
        }
    }

    pub fn with_progress_reporter(mut self, reporter: Box<dyn ProgressReporter>) -> Self {
        self.progress_reporter = Some(reporter);
        self
    }

    async fn report_progress(&self, current: u32, total: u32, phase: &str, detail: Option<String>) {
        if let Some(ref reporter) = self.progress_reporter {
            reporter
                .report(ProgressInfo {
                    current,
                    total,
                    phase: phase.to_string(),
                    detail,
                })
                .await;
        }
    }
}

#[async_trait]
impl ExecuteOperation for ExecuteOperationUseCase {
    async fn execute(&self, command: OperationCommand) -> Result<OperationResult, UseCaseError> {
        self.validate_command(&command)?;

        let now = self.clock.now();
        let operation_id = self.id_generator.generate();
        let target_folder = self.resolve_target_folder_async(&command).await?;

        let mut operation = Operation::new(
            operation_id,
            command.operation_type.clone(),
            command.source_paths.clone(),
            target_folder.clone(),
            command.target_paths.clone(),
            now,
        );

        operation
            .start()
            .map_err(|e| UseCaseError::Domain(e.to_string()))?;

        let total = command.source_paths.len() as u32;
        let mut total_files: u32 = 0;
        let mut total_bytes: u64 = 0;
        let mut last_error: Option<String> = None;

        for (idx, source) in command.source_paths.iter().enumerate() {
            self.report_progress(idx as u32, total, "processing", Some(source.to_string()))
                .await;

            match self.execute_single(source, &command, &target_folder).await {
                Ok(bytes) => {
                    total_files += 1;
                    total_bytes += bytes;
                }
                Err(e) => {
                    last_error = Some(e.to_string());
                    break;
                }
            }
        }

        self.report_progress(total, total, "complete", None).await;

        if let Some(reason) = last_error {
            operation
                .fail(reason.clone())
                .map_err(|e| UseCaseError::Domain(e.to_string()))?;
            self.operation_repository
                .save(&operation)
                .await
                .map_err(|e| UseCaseError::RepositoryError(e.to_string()))?;
            return Err(UseCaseError::FileSystemError(reason));
        }

        operation
            .complete(total_files, total_bytes)
            .map_err(|e| UseCaseError::Domain(e.to_string()))?;
        self.operation_repository
            .save(&operation)
            .await
            .map_err(|e| UseCaseError::RepositoryError(e.to_string()))?;

        Ok(OperationResult {
            operation_id,
            state: OperationState::Completed {
                processed_files: total_files,
                bytes_processed: total_bytes,
            },
            processed_files: total_files,
            bytes_moved: total_bytes,
        })
    }
}

impl ExecuteOperationUseCase {
    async fn execute_single(
        &self,
        source: &AbsolutePath,
        command: &OperationCommand,
        target_folder: &Option<AbsolutePath>,
    ) -> Result<u64, UseCaseError> {
        match command.operation_type {
            OperationType::Move | OperationType::Copy => {
                // Check if source file still exists (re-move protection)
                if !self
                    .file_system
                    .exists(source)
                    .await
                    .map_err(|e| UseCaseError::FileSystemError(e.to_string()))?
                {
                    return Err(UseCaseError::FileSystemError(format!(
                        "Source file not found (may have been moved already): {}",
                        source
                    )));
                }

                // Skip files already in the target folder (same-folder protection)
                if let (Some(src_parent), Some(ref target)) = (source.parent(), target_folder) {
                    if src_parent == *target {
                        return Ok(0u64);
                    }
                }

                let dest = self.build_destination(source, target_folder)?;

                // Duplicate detection phase
                let dup_result = self
                    .duplicate_detector
                    .check_duplicate(source, &dest, &command.duplicate_check_mode)
                    .await
                    .map_err(|e| UseCaseError::FileSystemError(e.to_string()))?;

                // If duplicate found, apply overwrite policy
                if dup_result.exists {
                    match command.overwrite_policy {
                        OverwritePolicy::Skip => {
                            return Err(UseCaseError::Conflict(format!(
                                "Duplicate found ({} mode): {}",
                                match command.duplicate_check_mode {
                                    DuplicateCheckMode::Name => "name",
                                    DuplicateCheckMode::Size => "size",
                                    DuplicateCheckMode::Content => "content",
                                },
                                dest
                            )));
                        }
                        OverwritePolicy::Overwrite => {
                            // Proceed with the operation
                        }
                        OverwritePolicy::AutoRename => {
                            let resolved = self.unique_name(&dest).await?;
                            return match command.operation_type {
                                OperationType::Move => self
                                    .file_system
                                    .move_file(source, &resolved)
                                    .await
                                    .map_err(|e| UseCaseError::FileSystemError(e.to_string())),
                                OperationType::Copy => self
                                    .file_system
                                    .copy_file(source, &resolved)
                                    .await
                                    .map_err(|e| UseCaseError::FileSystemError(e.to_string())),
                                _ => unreachable!(),
                            };
                        }
                        OverwritePolicy::Ask => {
                            // In non-interactive mode (IPC from DLL), fall back to AutoRename
                            let resolved = self.unique_name(&dest).await?;
                            return match command.operation_type {
                                OperationType::Move => self
                                    .file_system
                                    .move_file(source, &resolved)
                                    .await
                                    .map_err(|e| UseCaseError::FileSystemError(e.to_string())),
                                OperationType::Copy => self
                                    .file_system
                                    .copy_file(source, &resolved)
                                    .await
                                    .map_err(|e| UseCaseError::FileSystemError(e.to_string())),
                                _ => unreachable!(),
                            };
                        }
                    }
                }

                // No duplicate or Overwrite policy — proceed
                match command.operation_type {
                    OperationType::Move => self
                        .file_system
                        .move_file(source, &dest)
                        .await
                        .map_err(|e| UseCaseError::FileSystemError(e.to_string())),
                    OperationType::Copy => self
                        .file_system
                        .copy_file(source, &dest)
                        .await
                        .map_err(|e| UseCaseError::FileSystemError(e.to_string())),
                    _ => unreachable!(),
                }
            }
            OperationType::Delete => self
                .file_system
                .delete_file(source)
                .await
                .map_err(|e| UseCaseError::FileSystemError(e.to_string()))
                .map(|_| 0u64),
            OperationType::Rename => {
                let new_path = command
                    .target_paths
                    .as_ref()
                    .and_then(|p| p.first())
                    .ok_or_else(|| {
                        UseCaseError::InvalidCommand("Rename requires target_paths".to_string())
                    })?;
                self.file_system
                    .rename_file(source, new_path)
                    .await
                    .map_err(|e| UseCaseError::FileSystemError(e.to_string()))
                    .map(|_| 0u64)
            }
        }
    }

    fn build_destination(
        &self,
        source: &AbsolutePath,
        target_folder: &Option<AbsolutePath>,
    ) -> Result<AbsolutePath, UseCaseError> {
        let folder = target_folder
            .as_ref()
            .ok_or_else(|| UseCaseError::InvalidCommand("Target folder is required".to_string()))?;
        let file_name = source
            .file_name()
            .ok_or_else(|| UseCaseError::InvalidCommand("Cannot extract file name".to_string()))?;
        Ok(folder.join(file_name))
    }

    async fn unique_name(&self, path: &AbsolutePath) -> Result<AbsolutePath, UseCaseError> {
        let file_name = path.file_name().map(|s| s.to_string()).unwrap_or_default();
        let ext = path
            .extension()
            .map(|e| format!(".{}", e))
            .unwrap_or_default();
        let parent = path.parent().unwrap_or_else(|| path.clone());

        let base_name = if ext.is_empty() {
            file_name
        } else {
            file_name[..file_name.len() - ext.len()].to_string()
        };

        for counter in 1..=1000 {
            let candidate = parent.join(format!("{} ({}){}", base_name, counter, ext));
            if !self
                .file_system
                .exists(&candidate)
                .await
                .map_err(|e| UseCaseError::FileSystemError(e.to_string()))?
            {
                return Ok(candidate);
            }
        }

        Err(UseCaseError::Internal(
            "Could not find unique filename after 1000 attempts".to_string(),
        ))
    }

    async fn resolve_target_folder_async(
        &self,
        command: &OperationCommand,
    ) -> Result<Option<AbsolutePath>, UseCaseError> {
        if let Some(folder_id) = &command.target_folder_id {
            let folder = self
                .configuration_repository
                .find_by_id(folder_id)
                .await
                .map_err(|e| UseCaseError::RepositoryError(e.to_string()))?
                .ok_or_else(|| UseCaseError::FolderNotFound(folder_id.to_string()))?;
            Ok(Some(folder.path))
        } else {
            Ok(None)
        }
    }

    fn validate_command(&self, command: &OperationCommand) -> Result<(), UseCaseError> {
        if command.source_paths.is_empty() {
            return Err(UseCaseError::InvalidCommand(
                "source_paths must not be empty".to_string(),
            ));
        }

        match command.operation_type {
            OperationType::Move | OperationType::Copy => {
                if command.target_folder_id.is_none() {
                    return Err(UseCaseError::InvalidCommand(
                        "Move/Copy requires target_folder_id".to_string(),
                    ));
                }
            }
            OperationType::Rename => {
                if command.target_paths.is_none() {
                    return Err(UseCaseError::InvalidCommand(
                        "Rename requires target_paths".to_string(),
                    ));
                }
            }
            OperationType::Delete => {}
        }

        Ok(())
    }
}
