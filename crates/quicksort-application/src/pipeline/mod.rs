//! Command processing pipeline.
//!
//! The pipeline provides a structured way to process incoming commands
//! (from Tauri, CLI, or IPC) through a series of stages:
//! 1. **Validation** – check that the command is well-formed.
//! 2. **Execution** – delegate to the appropriate Use Case.
//! 3. **Logging** – record the outcome for diagnostics and auditing.
//!
//! # Design Decisions
//! - Uses a simple synchronous chain (not async) because the pipeline itself
//!   is a lightweight orchestrator; all I/O happens inside the Use Cases.
//! - Each stage is a separate function to allow easy testing and replacement.
//! - Returns `Result<OperationResult, UseCaseError>` so that adapters can
//!   map errors to user-friendly messages or IPC responses.

use crate::dtos::{OperationCommand, OperationResult};
use crate::errors::UseCaseError;
use crate::ports::inbound::ExecuteOperation;

/// Runs the full command processing pipeline.
///
/// This is the only public function in the pipeline module. Adapters should
/// call it with the command received from the user or external system.
pub fn run_pipeline(
    command: OperationCommand,
    execute_use_case: &dyn ExecuteOperation,
) -> Result<OperationResult, UseCaseError> {
    // Stage 1: Validate the command
    validate_command(&command)?;

    // Stage 2: Execute the command via the Use Case
    let result = execute_use_case.execute(command)?;

    // Stage 3: Log the outcome (in a real system, this might write to a file,
    // send to a monitoring system, or emit a domain event)
    log_outcome(&result);

    Ok(result)
}

// ---------------------------------------------------------------------------
// Private pipeline stages
// ---------------------------------------------------------------------------

/// Validates the incoming command.
///
/// # Current Validation Rules
/// - The command must have at least one source path (except for Delete,
///   which should have at least one path as well, but that's enforced by
///   the Use Case).
/// - The `overwrite_policy` is always valid because it's an enum; no
///   further checking is needed.
///
/// # Future Extensions
/// - Path traversal prevention (reject paths containing "..")
/// - Size limits (reject operations on files larger than a threshold)
/// - Rate limiting (reject if too many operations are in progress)
fn validate_command(command: &OperationCommand) -> Result<(), UseCaseError> {
    // Ensure at least one source path is provided.
    if command.source_paths.is_empty() {
        return Err(UseCaseError::InvalidCommand(
            "At least one source path is required".to_string(),
        ));
    }

    // OLD: no path traversal check
    // NEW: basic path traversal prevention – reject paths that try to escape
    // the allowed directories via ".." sequences.
    for path in &command.source_paths {
        // Check for ".." components which could be used to traverse directories
        if path.components().any(|c| c == std::path::Component::ParentDir) {
            return Err(UseCaseError::InvalidCommand(format!(
                "Path traversal detected in source path: {}",
                path.display()
            )));
        }
    }

    Ok(())
}

/// Logs the outcome of a successfully executed operation.
///
/// In the current implementation, this simply writes to the tracing log.
/// In the future, this could:
/// - Persist an audit record to the `OperationRepository`.
/// - Emit a domain event via an event bus.
/// - Send a notification to the user (e.g., toast message).
fn log_outcome(result: &OperationResult) {
    tracing::info!(
        operation_id = %result.operation_id,
        files_processed = result.processed_files,
        bytes_moved = result.bytes_moved,
        "Operation completed successfully"
    );
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dtos::{OperationType, OverwritePolicy};
    use quicksort_domain::WindowsPath;

    /// A simple mock that always succeeds.
    struct MockExecuteOperation;

    #[async_trait::async_trait]
    impl ExecuteOperation for MockExecuteOperation {
        async fn execute(&self, command: OperationCommand) -> Result<OperationResult, UseCaseError> {
            Ok(OperationResult {
                operation_id: quicksort_domain::OperationId::new(),
                state: quicksort_domain::OperationState::Completed {
                    processed_files: command.source_paths.len() as u32,
                    bytes_processed: 0,
                },
                processed_files: command.source_paths.len() as u32,
                bytes_moved: 0,
            })
        }
    }

    /// A simple mock that always fails.
    struct FailingMockExecuteOperation;

    #[async_trait::async_trait]
    impl ExecuteOperation for FailingMockExecuteOperation {
        async fn execute(&self, _command: OperationCommand) -> Result<OperationResult, UseCaseError> {
            Err(UseCaseError::FileSystemError("mock error".to_string()))
        }
    }

    /// Helper to create a test command.
    fn test_command() -> OperationCommand {
        OperationCommand {
            operation_type: OperationType::Move,
            source_paths: vec![WindowsPath::new("C:\\test.txt").unwrap()],
            target_folder_id: Some("folder-1".to_string()),
            overwrite_policy: OverwritePolicy::Skip,
            target_paths: None,
        }
    }

    #[test]
    fn test_validation_empty_sources() {
        let mut cmd = test_command();
        cmd.source_paths = vec![];
        let result = validate_command(&cmd);
        assert!(matches!(result, Err(UseCaseError::InvalidCommand(_))));
    }

    #[test]
    fn test_validation_path_traversal() {
        let mut cmd = test_command();
        cmd.source_paths = vec![WindowsPath::new("C:\\..\\secret.txt").unwrap()];
        let result = validate_command(&cmd);
        assert!(matches!(result, Err(UseCaseError::InvalidCommand(_))));
    }

    #[test]
    fn test_validation_valid() {
        let cmd = test_command();
        let result = validate_command(&cmd);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_pipeline_success() {
        let mock = MockExecuteOperation;
        let cmd = test_command();
        let result = run_pipeline(cmd, &mock);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_pipeline_execution_error() {
        let mock = FailingMockExecuteOperation;
        let cmd = test_command();
        let result = run_pipeline(cmd, &mock);
        assert!(matches!(result, Err(UseCaseError::FileSystemError(_))));
    }
}