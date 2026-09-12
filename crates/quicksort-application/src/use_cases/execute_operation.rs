use crate::dtos::{OperationCommand, OperationResult, OverwritePolicy};
use crate::errors::UseCaseError;
use crate::ports::inbound::ExecuteOperation;
use crate::ports::outbound::{
    Clock, ConfigurationRepository, DuplicateDetectionPort, FileSystem, IdGenerator,
    OperationRepository, ProgressCallback, ProgressInfo, ProgressReporter,
};
use async_trait::async_trait;
use quicksort_domain::{AbsolutePath, Operation, OperationState, OperationType};

/// Result of the pre-flight disk-space check plus the byte weights used to
/// build the progress scale (re-QA 12.09.2026, item 3.5).
///
/// `copies` mirrors `source_paths`: `true` marks a source that is actually
/// copied during the operation (Copy sources and cross-volume Move sources).
/// Same-volume moves, renames and deletes do not copy and report no progress.
struct SpaceCheck {
    total_bytes: u64,
    copies: Vec<bool>,
}

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

    fn report_progress(&self, current: u64, total: u64, phase: &str, detail: Option<String>) {
        if let Some(ref reporter) = self.progress_reporter {
            reporter.report(ProgressInfo {
                current,
                total,
                phase: phase.to_string(),
                detail,
            });
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

        // Propagate the operation origin and its trace id from the command so
        // the Operations UI and audit logs can link intent to execution
        // (spec #15, release 0.2.6).
        operation.source = command.source;
        operation.correlation_id = command.correlation_id;

        operation
            .start()
            .map_err(|e| UseCaseError::Domain(e.to_string()))?;

        let mut total_files: u32 = 0;
        let mut history_bytes: u64 = 0;
        let mut progress_current: u64 = 0;
        let mut last_error: Option<UseCaseError> = None;

        // Pre-flight check: fail BEFORE moving/copying starts when the target
        // volume lacks the space an operation needs (re-QA 12.09.2026 D3).
        let space = match self.check_disk_space(&command, &target_folder).await {
            Ok(s) => s,
            Err(e) => {
                operation
                    .fail(e.to_string())
                    .map_err(|err| UseCaseError::Domain(err.to_string()))?;
                self.operation_repository
                    .save(&operation)
                    .await
                    .map_err(|err| UseCaseError::RepositoryError(err.to_string()))?;
                return Err(e);
            }
        };

        // Byte-based progress scale when the operation moves real bytes;
        // fall back to a per-source scale for deletes, renames and
        // same-volume moves (re-QA 12.09.2026, item 3.5).
        let byte_mode = space.total_bytes > 0;
        let total_scale = if byte_mode {
            space.total_bytes
        } else {
            command.source_paths.len() as u64
        };

        for (idx, source) in command.source_paths.iter().enumerate() {
            if !byte_mode {
                self.report_progress(
                    idx as u64,
                    total_scale,
                    "processing",
                    Some(source.to_string()),
                );
            }

            // Snapshot of the bytes already accounted by earlier sources. The
            // FileSystem callback reports its per-file ticks relative to this
            // base, keeping the progress bar monotonic across sources.
            let base = progress_current;
            let cb = move |copied: u64, name: &str| {
                self.report_progress(
                    base + copied,
                    total_scale,
                    "copying",
                    Some(name.to_string()),
                );
            };
            let on_progress: Option<&(dyn Fn(u64, &str) + Send + Sync)> =
                if byte_mode && space.copies[idx] {
                    Some(&cb as &(dyn Fn(u64, &str) + Send + Sync))
                } else {
                    None
                };

            match self
                .execute_single(source, &command, &target_folder, on_progress)
                .await
            {
                Ok(bytes) => {
                    total_files += 1;
                    history_bytes += bytes;
                    if space.copies[idx] {
                        progress_current += bytes;
                    }
                }
                Err(e) => {
                    // Keep processing the remaining items so a single
                    // failure (e.g. a locked file or an unsupported path)
                    // does not abort the whole bundle. The last error is
                    // reported once the loop finishes.
                    last_error = Some(e);
                }
            }
        }

        self.report_progress(total_scale, total_scale, "complete", None);

        if let Some(error) = last_error {
            let reason = error.to_string();
            operation.record_progress(total_files, history_bytes);
            operation
                .fail(reason.clone())
                .map_err(|e| UseCaseError::Domain(e.to_string()))?;
            self.operation_repository
                .save(&operation)
                .await
                .map_err(|e| UseCaseError::RepositoryError(e.to_string()))?;
            return Err(error);
        }

        operation
            .complete(total_files, history_bytes)
            .map_err(|e| UseCaseError::Domain(e.to_string()))?;
        self.operation_repository
            .save(&operation)
            .await
            .map_err(|e| UseCaseError::RepositoryError(e.to_string()))?;

        Ok(OperationResult {
            operation_id,
            state: OperationState::Completed {
                processed_files: total_files,
                bytes_processed: history_bytes,
            },
            processed_files: total_files,
            bytes_moved: history_bytes,
        })
    }
}

impl ExecuteOperationUseCase {
    async fn execute_single(
        &self,
        source: &AbsolutePath,
        command: &OperationCommand,
        target_folder: &Option<AbsolutePath>,
        on_progress: ProgressCallback<'_>,
    ) -> Result<u64, UseCaseError> {
        match command.operation_type {
            OperationType::Move | OperationType::Copy => {
                // Check if source file still exists (re-move protection)
                if !self.file_system.exists(source).await? {
                    return Err(UseCaseError::FileSystemError(format!(
                        "Source file not found (may have been moved already): {}",
                        source
                    )));
                }

                // Reject moving/copying into the same folder the entity already
                // lives in. The comparison is case-insensitive on Windows
                // (PathBuf equality is byte/case-sensitive even on Windows, so
                // a source and target differing only in case would otherwise
                // be treated as different).
                if let (Some(src_parent), Some(ref target)) = (source.parent(), target_folder) {
                    let same = if cfg!(target_os = "windows") {
                        src_parent
                            .as_str()
                            .and_then(|s| {
                                target
                                    .as_str()
                                    .map(|t| (s.to_lowercase(), t.to_lowercase()))
                            })
                            .is_some_and(|(s, t)| s == t)
                    } else {
                        src_parent == *target
                    };
                    if same {
                        return Err(UseCaseError::Conflict(format!(
                            "Source is already in the target folder: {}",
                            source
                        )));
                    }
                }

                let dest = self.build_destination(source, target_folder)?;

                // Destination-existence phase. The overwrite policy must apply
                // to ANY name collision at the destination, not only to what the
                // duplicate checker reports: in Size mode a same-named file with
                // a DIFFERENT size yields exists=false, which previously let the
                // operation silently overwrite the destination (QA report
                // 07.09.2026, lines 84-89).
                let dest_exists = self.file_system.exists(&dest).await?;

                // Still run the duplicate check for parity with prior logs.
                let _ = self
                    .duplicate_detector
                    .check_duplicate(source, &dest, &command.duplicate_check_mode)
                    .await
                    .map_err(|e| UseCaseError::FileSystemError(e.to_string()))?;

                if dest_exists {
                    match command.overwrite_policy {
                        OverwritePolicy::Skip => {
                            return Err(UseCaseError::Conflict(format!(
                                "Destination already exists: {}",
                                dest
                            )));
                        }
                        OverwritePolicy::Overwrite => {
                            // Proceed with the operation (replaces the destination)
                        }
                        OverwritePolicy::AutoRename => {
                            let resolved = self.unique_name(&dest).await?;
                            return match command.operation_type {
                                OperationType::Move => {
                                    self.perform_move(source, &resolved, on_progress).await
                                }
                                OperationType::Copy => {
                                    self.perform_copy(source, &resolved, on_progress).await
                                }
                                _ => unreachable!(),
                            };
                        }
                        OverwritePolicy::Ask => {
                            // In non-interactive mode (IPC from DLL), fall back to AutoRename
                            let resolved = self.unique_name(&dest).await?;
                            return match command.operation_type {
                                OperationType::Move => {
                                    self.perform_move(source, &resolved, on_progress).await
                                }
                                OperationType::Copy => {
                                    self.perform_copy(source, &resolved, on_progress).await
                                }
                                _ => unreachable!(),
                            };
                        }
                    }
                }

                // No destination conflict (or Overwrite policy) — proceed
                match command.operation_type {
                    OperationType::Move => self.perform_move(source, &dest, on_progress).await,
                    OperationType::Copy => self.perform_copy(source, &dest, on_progress).await,
                    _ => unreachable!(),
                }
            }
            OperationType::Delete => self.file_system.delete_file(source).await.map(|_| 0u64),
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
                    .map(|_| 0u64)
            }
        }
    }

    async fn check_disk_space(
        &self,
        command: &OperationCommand,
        target_folder: &Option<AbsolutePath>,
    ) -> Result<SpaceCheck, UseCaseError> {
        let target = match target_folder {
            Some(t) => t,
            None => {
                return Ok(SpaceCheck {
                    total_bytes: 0,
                    copies: vec![false; command.source_paths.len()],
                });
            }
        };

        let mut needed: u64 = 0;
        let mut total_bytes: u64 = 0;
        let mut copies = Vec::with_capacity(command.source_paths.len());

        for source in &command.source_paths {
            match command.operation_type {
                OperationType::Copy => {
                    let size = self.source_size(source).await?;
                    copies.push(true);
                    total_bytes += size;
                    needed += size;
                }
                OperationType::Move => {
                    // Cross-volume moves copy first, so they need space for
                    // the copy; same-volume moves are renames and need none.
                    let cross_volume = !self.file_system.is_same_volume(source, target).await?;
                    copies.push(cross_volume);
                    let size = self.source_size(source).await?;
                    total_bytes += size;
                    if cross_volume {
                        needed += size;
                    }
                }
                OperationType::Delete | OperationType::Rename => {
                    copies.push(false);
                }
            }
        }

        if needed > 0 {
            let available = self.file_system.available_space(target).await?;
            if needed > available {
                return Err(UseCaseError::InsufficientDiskSpace {
                    need: needed,
                    available,
                });
            }
        }

        Ok(SpaceCheck {
            total_bytes,
            copies,
        })
    }

    async fn source_size(&self, source: &AbsolutePath) -> Result<u64, UseCaseError> {
        // A source that vanished since the operation was queued weighs zero:
        // execute_single reports the missing-file error with its specific
        // message instead of failing the whole pre-flight.
        if !self.file_system.exists(source).await? {
            return Ok(0);
        }
        if self.file_system.is_dir(source).await? {
            Ok(self.file_system.folder_metadata(source).await?.total_size)
        } else {
            self.file_system.get_file_size(source).await
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

    async fn perform_move(
        &self,
        from: &AbsolutePath,
        to: &AbsolutePath,
        on_progress: ProgressCallback<'_>,
    ) -> Result<u64, UseCaseError> {
        if self.file_system.is_dir(from).await? {
            self.file_system.move_tree(from, to, on_progress).await
        } else {
            self.file_system.move_file(from, to, on_progress).await
        }
    }

    async fn perform_copy(
        &self,
        from: &AbsolutePath,
        to: &AbsolutePath,
        on_progress: ProgressCallback<'_>,
    ) -> Result<u64, UseCaseError> {
        if self.file_system.is_dir(from).await? {
            self.file_system.copy_tree(from, to, on_progress).await
        } else {
            self.file_system.copy_file(from, to, on_progress).await
        }
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
            // `exists()` already returns a `UseCaseError`; re-wrapping it in
            // `FileSystemError` would double the "File system error:" prefix.
            if !self.file_system.exists(&candidate).await? {
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
