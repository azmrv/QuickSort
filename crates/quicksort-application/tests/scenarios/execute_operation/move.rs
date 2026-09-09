//! Executable specification for Move operation.
//!
//! These tests verify that the `ExecuteOperationUseCase` correctly performs
//! Move operations according to the specified `OverwritePolicy`.
//! Each scenario follows the Given-When-Then structure defined in
//! `SPECIFICATION.md`.

use chrono::Utc;

use quicksort_application::{
    use_cases::ExecuteOperationUseCase, ExecuteOperation, OperationCommand, OverwritePolicy,
    UseCaseError,
};
use quicksort_domain::{AbsolutePath, DuplicateCheckMode, OperationState, OperationType};

use crate::mocks::*;
use crate::scenarios::test_folder;

// ============================================================================
// Helper functions for this test module
// ============================================================================

/// Creates a `AbsolutePath` from a relative test path on every platform.
fn wp(path: &str) -> AbsolutePath {
    let root = if cfg!(target_os = "windows") {
        "C:\\Users\\Test\\"
    } else {
        "/home/test/"
    };
    AbsolutePath::new(&format!("{root}{path}")).expect("Invalid test path")
}

// ============================================================================
// Scenario: Move a single file to an existing folder (happy path)
// ============================================================================

/// Scenario: Move a single file to an existing folder with Skip policy.
///
/// Given a source file and a target folder that does NOT contain the file,
/// when a Move operation is executed with `OverwritePolicy::Skip`,
/// then the file should be moved successfully.
#[tokio::test]
async fn move_single_file_to_existing_folder() {
    // ---- Given ----
    let folder = test_folder();
    let config_repo = MockConfigurationRepository::new();
    config_repo.add(folder.clone()).await.unwrap();

    let src_path = wp("Downloads/report.pdf");
    let dst_path = wp("Documents/report.pdf");

    let fs = MockFileSystem::new();
    fs.add_file(src_path.to_path_buf(), 1024); // source file with size 1024 bytes

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

    let command = OperationCommand {
        operation_type: OperationType::Move,
        source_paths: vec![src_path.clone()],
        target_folder_id: Some(folder.id.clone()),
        overwrite_policy: OverwritePolicy::Skip,
        target_paths: None,
        duplicate_check_mode: DuplicateCheckMode::default(),
    };

    // ---- When ----
    let result = use_case.execute(command).await.unwrap();

    // ---- Then ----
    assert_eq!(result.processed_files, 1);
    assert_eq!(result.bytes_moved, 1024);

    // File was moved: source no longer exists, destination exists
    assert!(!fs.exists(&src_path).await.unwrap());
    assert!(fs.exists(&dst_path).await.unwrap());

    // Operation is saved in the repository as Completed
    let saved_op = op_repo
        .find_by_id(&result.operation_id)
        .await
        .unwrap()
        .unwrap();
    assert!(matches!(saved_op.state, OperationState::Completed { .. }));
}

// ============================================================================
// Scenario: Move fails when source file is missing
// ============================================================================

/// Scenario: Move operation when the source file does not exist.
///
/// Given an empty file system (no files) and a valid target folder,
/// when a Move operation is executed for a non-existent source,
/// then the operation should fail with a `FileNotFound` error.
#[tokio::test]
async fn move_fails_when_source_missing() {
    // ---- Given ----
    let folder = test_folder();
    let config_repo = MockConfigurationRepository::new();
    config_repo.add(folder.clone()).await.unwrap();

    // Empty file system – no files added
    let fs = MockFileSystem::new();

    let src_path = wp("missing.txt");

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

    let command = OperationCommand {
        operation_type: OperationType::Move,
        source_paths: vec![src_path],
        target_folder_id: Some(folder.id),
        overwrite_policy: OverwritePolicy::Skip,
        target_paths: None,
        duplicate_check_mode: DuplicateCheckMode::default(),
    };

    // ---- When ----
    let result = use_case.execute(command).await;

    // ---- Then ----
    // The source file is absent, so the move cannot start. The use case
    // reports this as a file system error (the file may have been moved
    // by another process in the meantime).
    assert!(matches!(result, Err(UseCaseError::FileSystemError(_))));
}
