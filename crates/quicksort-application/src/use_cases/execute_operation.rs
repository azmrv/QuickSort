use crate::dtos::{OperationCommand, OperationResult, OverwritePolicy};
use crate::errors::UseCaseError;
use crate::ports::inbound::ExecuteOperation;
use crate::ports::outbound::{
    Clock, ConfigurationRepository, FileSystem, IdGenerator, OperationRepository,
};
use async_trait::async_trait;
use quicksort_domain::{Operation, OperationState, OperationType, WindowsPath};

pub struct ExecuteOperationUseCase {
    operation_repository: Box<dyn OperationRepository>,
    configuration_repository: Box<dyn ConfigurationRepository>,
    file_system: Box<dyn FileSystem>,
    id_generator: Box<dyn IdGenerator>,
    clock: Box<dyn Clock>,
}

impl ExecuteOperationUseCase {
    pub fn new(
        operation_repository: Box<dyn OperationRepository>,
        configuration_repository: Box<dyn ConfigurationRepository>,
        file_system: Box<dyn FileSystem>,
        id_generator: Box<dyn IdGenerator>,
        clock: Box<dyn Clock>,
    ) -> Self {
        Self {
            operation_repository,
            configuration_repository,
            file_system,
            id_generator,
            clock,
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

        let mut total_files: u32 = 0;
        let mut total_bytes: u64 = 0;
        let mut last_error: Option<String> = None;

        for source in &command.source_paths {
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
        source: &WindowsPath,
        command: &OperationCommand,
        target_folder: &Option<WindowsPath>,
    ) -> Result<u64, UseCaseError> {
        match command.operation_type {
            OperationType::Move => {
                let dest = self.build_destination(source, target_folder)?;
                let resolved = self
                    .resolve_conflict(&dest, &command.overwrite_policy)
                    .await?;
                self.file_system
                    .move_file(source, &resolved)
                    .await
                    .map_err(|e| UseCaseError::FileSystemError(e.to_string()))
            }
            OperationType::Copy => {
                let dest = self.build_destination(source, target_folder)?;
                let resolved = self
                    .resolve_conflict(&dest, &command.overwrite_policy)
                    .await?;
                self.file_system
                    .copy_file(source, &resolved)
                    .await
                    .map_err(|e| UseCaseError::FileSystemError(e.to_string()))
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
        source: &WindowsPath,
        target_folder: &Option<WindowsPath>,
    ) -> Result<WindowsPath, UseCaseError> {
        let folder = target_folder
            .as_ref()
            .ok_or_else(|| UseCaseError::InvalidCommand("Target folder is required".to_string()))?;
        let file_name = source
            .file_name()
            .ok_or_else(|| UseCaseError::InvalidCommand("Cannot extract file name".to_string()))?;
        Ok(folder.join(file_name))
    }

    async fn resolve_conflict(
        &self,
        path: &WindowsPath,
        policy: &OverwritePolicy,
    ) -> Result<WindowsPath, UseCaseError> {
        if !self
            .file_system
            .exists(path)
            .await
            .map_err(|e| UseCaseError::FileSystemError(e.to_string()))?
        {
            return Ok(path.clone());
        }

        match policy {
            OverwritePolicy::Overwrite => Ok(path.clone()),
            OverwritePolicy::Skip => Err(UseCaseError::Conflict(format!(
                "File already exists: {}",
                path
            ))),
            OverwritePolicy::AutoRename => self.unique_name(path).await,
            OverwritePolicy::Ask => self.unique_name(path).await,
        }
    }

    async fn unique_name(&self, path: &WindowsPath) -> Result<WindowsPath, UseCaseError> {
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
    ) -> Result<Option<WindowsPath>, UseCaseError> {
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
