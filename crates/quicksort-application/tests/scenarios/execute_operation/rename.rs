//! Executable specification for Rename operation.
//!
//! These tests verify that the `ExecuteOperationUseCase` correctly performs
//! Rename operations. Each scenario follows the Given-When-Then structure
//! defined in `SPECIFICATION.md`.

use chrono::Utc;

use quicksort_application::{
    use_cases::ExecuteOperationUseCase, ExecuteOperation, OperationCommand, OverwritePolicy,
    UseCaseError,
};
use quicksort_domain::{AbsolutePath, DuplicateCheckMode, OperationState, OperationType};

use crate::mocks::*;

// ============================================================================
// Helper functions for this test module
// ============================================================================

/// Creates a `AbsolutePath` from a string for test purposes.
fn wp(path: &str) -> AbsolutePath {
    AbsolutePath::new(path).expect("Invalid test path")
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
    fs.add_file(old_path.to_path_buf(), 1024); // source file exists

    let op_repo = MockOperationRepository::new();
    let id_gen = MockIdGenerator::new();
    let clock = MockClock::new(Utc::now());

    let use_case = ExecuteOperationUseCase::new(
        Box::new(op_repo.clone()),
        Box::new(config_repo),
        Box::new(fs.clone()),
        Box::new(id_gen),
        Box::new(clock),
        Box::new(MockDuplicateDetector),
    );

    // For Rename, `source_paths` holds the old path(s) and
    // `target_paths` holds the corresponding new path(s).
    let command = OperationCommand {
        operation_type: OperationType::Rename,
        source_paths: vec![old_path.clone()],
        target_folder_id: None,
        overwrite_policy: OverwritePolicy::Skip,
        target_paths: Some(vec![new_path.clone()]),
        duplicate_check_mode: DuplicateCheckMode::default(),
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
    let saved_op = op_repo
        .find_by_id(&result.operation_id)
        .await
        .unwrap()
        .unwrap();
    assert!(matches!(saved_op.state, OperationState::Completed { .. }));
}

// ============================================================================
// Scenario: Rename fails when source and target counts mismatch
// ============================================================================

/// Scenario: Rename with a mismatched second source path.
///
/// Given a Rename command with more source paths than target paths,
/// when the operation is executed,
/// then it should fail because the unmatched source file does not exist.
#[tokio::test]
async fn rename_mismatched_counts() {
    // ---- Given ----
    let config_repo = MockConfigurationRepository::new();

    let old_path = wp("C:\\old.txt");
    let new_path = wp("C:\\new.txt");

    let fs = MockFileSystem::new();
    fs.add_file(old_path.to_path_buf(), 512);

    let use_case = ExecuteOperationUseCase::new(
        Box::new(MockOperationRepository::new()),
        Box::new(config_repo),
        Box::new(fs),
        Box::new(MockIdGenerator::new()),
        Box::new(MockClock::new(Utc::now())),
        Box::new(MockDuplicateDetector),
    );

    // Two source paths, but only one target path – the second source
    // (C:\second.txt) does not exist on the file system.
    let command = OperationCommand {
        operation_type: OperationType::Rename,
        source_paths: vec![old_path.clone(), wp("C:\\second.txt")],
        target_folder_id: None,
        overwrite_policy: OverwritePolicy::Skip,
        target_paths: Some(vec![new_path.clone()]),
        duplicate_check_mode: DuplicateCheckMode::default(),
    };

    // ---- When ----
    let result = use_case.execute(command).await;

    // ---- Then ----
    assert!(matches!(result, Err(UseCaseError::FileNotFound(_))));
}
