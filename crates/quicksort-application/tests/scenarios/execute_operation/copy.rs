//! Executable specification for Copy operation.
//!
//! These tests verify that the `ExecuteOperationUseCase` correctly performs
//! Copy operations according to the specified `OverwritePolicy`.
//! Each scenario follows the Given-When-Then structure defined in
//! `SPECIFICATION.md`.

use chrono::Utc;

use quicksort_application::{
    use_cases::ExecuteOperationUseCase, ExecuteOperation, OperationCommand, OverwritePolicy,
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
// Scenario: Copy a single file to an existing folder
// ============================================================================

/// Scenario: Copy a single file to an existing folder with Skip policy.
///
/// Given a source file and a target folder that does NOT contain the file,
/// when a Copy operation is executed with `OverwritePolicy::Skip`,
/// then the file should be copied and both source and destination exist.
#[tokio::test]
async fn copy_single_file_to_existing_folder() {
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
        operation_type: OperationType::Copy,
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

    // Source file must still exist after copy
    assert!(fs.exists(&src_path).await.unwrap());
    // Destination file must now exist
    assert!(fs.exists(&dst_path).await.unwrap());

    // Operation should be saved in the repository
    let saved_op = op_repo
        .find_by_id(&result.operation_id)
        .await
        .unwrap()
        .unwrap();
    assert!(matches!(saved_op.state, OperationState::Completed { .. }));
}

// ============================================================================
// Scenario: Copy with conflict and AutoRename policy
// ============================================================================

/// Scenario: Copy with conflict and AutoRename policy.
///
/// Given a target folder that already contains "report.pdf",
/// when a Copy operation is executed with `OverwritePolicy::AutoRename`,
/// then the copied file should be renamed to avoid collision.
#[tokio::test]
async fn copy_with_conflict_auto_rename() {
    // ---- Given ----
    let folder = test_folder();
    let config_repo = MockConfigurationRepository::new();
    config_repo.add(folder.clone()).await.unwrap();

    let src_path = wp("Downloads/report.pdf");
    let dst_existing = wp("Documents/report.pdf");
    let dst_renamed = wp("Documents/report (1).pdf");

    let fs = MockFileSystem::new();
    fs.add_file(src_path.to_path_buf(), 1024); // source
    fs.add_file(dst_existing.to_path_buf(), 2048); // existing destination (conflict)

    let op_repo = MockOperationRepository::new();
    let use_case = ExecuteOperationUseCase::new(
        Box::new(op_repo.clone()),
        Box::new(config_repo),
        Box::new(fs.clone()),
        Box::new(MockIdGenerator::new()),
        Box::new(MockClock::new(Utc::now())),
        Box::new(MockDuplicateDetector),
    );

    let command = OperationCommand {
        operation_type: OperationType::Copy,
        source_paths: vec![src_path.clone()],
        target_folder_id: Some(folder.id.clone()),
        overwrite_policy: OverwritePolicy::AutoRename,
        target_paths: None,
        duplicate_check_mode: DuplicateCheckMode::default(),
    };

    // ---- When ----
    let result = use_case.execute(command).await.unwrap();

    // ---- Then ----
    assert_eq!(result.processed_files, 1);
    assert!(fs.exists(&src_path).await.unwrap()); // source still exists
    assert!(fs.exists(&dst_existing).await.unwrap()); // original destination unchanged
    assert!(fs.exists(&dst_renamed).await.unwrap()); // renamed copy created
}
