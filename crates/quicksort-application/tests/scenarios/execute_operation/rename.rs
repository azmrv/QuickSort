//! Executable specification for Rename operation.
//!
//! These tests verify that the `ExecuteOperationUseCase` correctly performs
//! Rename operations. Each scenario follows the Given-When-Then structure
//! defined in `SPECIFICATION.md`.

use std::sync::Arc;
use chrono::Utc;

use quicksort_domain::{OperationType, OperationState, WindowsPath};
use quicksort_application::{
    ExecuteOperation, OperationCommand, OverwritePolicy, UseCaseError,
    use_cases::ExecuteOperationUseCase,
};

use crate::mocks::*;

// ============================================================================
// Helper functions for this test module
// ============================================================================

/// Creates a `WindowsPath` from a string for test purposes.
fn wp(path: &str) -> WindowsPath {
    WindowsPath::new(path).expect("Invalid test path")
}

// ============================================================================
// Scenario: Rename a single file (happy path)
// ============================================================================

/// Scenario: Rename a file from an old name to a new name.
///
/// Given a source file that exists on the file system,
/// when a Rename operation is executed with a valid target path,
/// then the file should be renamed and the operation recorded.
#[tokio::test]
async fn rename_single_file() {
    // ---- Given ----
    // Rename does not require a target folder, so the config repo can be empty
    let config_repo = MockConfigurationRepository::new();

    let old_path = wp("C:\\Users\\Test\\Downloads\\old_name.txt");
    let new_path = wp("C:\\Users\\Test\\Downloads\\new_name.txt");

    let fs = MockFileSystem::new();
    fs.add_file(old_path.to_path_buf(), 1024);  // source file exists

    let op_repo = MockOperationRepository::new();
    let id_gen = MockIdGenerator::new();
    let clock = MockClock::new(Utc::now());

    let use_case = ExecuteOperationUseCase::new(
        Arc::new(config_repo),
        Arc::new(op_repo.clone()),
        Arc::new(fs.clone()),
        Arc::new(id_gen),
        Arc::new(clock),
        Arc::new(MockConflictResolver),
    );

    // For Rename, `source_paths` holds the old path(s) and
    // `target_paths` holds the corresponding new path(s).
    let command = OperationCommand {
        operation_type: OperationType::Rename,
        source_paths: vec![old_path.clone()],
        target_folder_id: None,                     // not used for Rename
        overwrite_policy: OverwritePolicy::Skip,    // not relevant here
        target_paths: Some(vec![new_path.clone()]),
    };

    // ---- When ----
    let result = use_case.execute(command).await.unwrap();

    // ---- Then ----
    assert_eq!(result.processed_files, 1);
    // Rename does not report a byte count (the operation records 0)
    assert_eq!(result.bytes_moved, 0);

    // Old file no longer exists, new file exists
    assert!(!fs.exists(&old_path).await.unwrap());
    assert!(fs.exists(&new_path).await.unwrap());

    // Operation is saved in the repository as Completed
    let saved_op = op_repo.find_by_id(&result.operation_id).await.unwrap().unwrap();
    assert!(matches!(saved_op.state, OperationState::Completed { .. }));
}

// ============================================================================
// Scenario: Rename fails when source and target counts mismatch
// ============================================================================

/// Scenario: Rename operation with mismatched source and target path counts.
///
/// Given a Rename command where the number of source paths does not equal
/// the number of target paths,
/// when the operation is executed,
/// then it should fail with an `InvalidCommand` error.
#[tokio::test]
async fn rename_mismatched_counts() {
    // ---- Given ----
    let config_repo = MockConfigurationRepository::new();

    let old_path = wp("C:\\old.txt");
    let new_path = wp("C:\\new.txt");

    let fs = MockFileSystem::new();
    fs.add_file(old_path.to_path_buf(), 512);

    let use_case = ExecuteOperationUseCase::new(
        Arc::new(config_repo),
        Arc::new(MockOperationRepository::new()),
        Arc::new(fs),
        Arc::new(MockIdGenerator::new()),
        Arc::new(MockClock::new(Utc::now())),
        Arc::new(MockConflictResolver),
    );

    // Two source paths, but only one target path – mismatch
    let command = OperationCommand {
        operation_type: OperationType::Rename,
        source_paths: vec![old_path.clone(), wp("C:\\second.txt")],
        target_folder_id: None,
        overwrite_policy: OverwritePolicy::Skip,
        target_paths: Some(vec![new_path.clone()]),
    };

    // ---- When ----
    let result = use_case.execute(command).await;

    // ---- Then ----
    assert!(matches!(result, Err(UseCaseError::InvalidCommand(_))));
}