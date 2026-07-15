//! Executable specification for Copy operation.
//!
//! These tests verify that the `ExecuteOperationUseCase` correctly performs
//! Copy operations according to the specified `OverwritePolicy`.
//! Each scenario follows the Given-When-Then structure defined in
//! `SPECIFICATION.md`.

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

/// Creates a test folder entity with default values.
fn test_folder() -> quicksort_domain::Folder {
    quicksort_domain::Folder {
        id: quicksort_domain::FolderId::from_string("folder-test"),
        name: "Test Folder".to_string(),
        path: wp("C:\\Users\\Test\\Documents"),
        favorite: false,
        order: 0,
        stats: Default::default(),
    }
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

    let src_path = wp("C:\\Users\\Test\\Downloads\\report.pdf");
    let dst_path = wp("C:\\Users\\Test\\Documents\\report.pdf");

    let fs = MockFileSystem::new();
    fs.add_file(src_path.to_path_buf(), 1024);  // source file with size 1024 bytes

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

    let command = OperationCommand {
        operation_type: OperationType::Copy,
        source_paths: vec![src_path.clone()],
        target_folder_id: Some(folder.id.clone()),
        overwrite_policy: OverwritePolicy::Skip,
        target_paths: None,   // not needed for Copy
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
    let saved_op = op_repo.find_by_id(&result.operation_id).await.unwrap().unwrap();
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

    let src_path = wp("C:\\Users\\Test\\Downloads\\report.pdf");
    let dst_existing = wp("C:\\Users\\Test\\Documents\\report.pdf");
    let dst_renamed = wp("C:\\Users\\Test\\Documents\\report (1).pdf");

    let fs = MockFileSystem::new();
    fs.add_file(src_path.to_path_buf(), 1024);      // source
    fs.add_file(dst_existing.to_path_buf(), 2048);  // existing destination (conflict)

    let op_repo = MockOperationRepository::new();
    let use_case = ExecuteOperationUseCase::new(
        Arc::new(config_repo),
        Arc::new(op_repo.clone()),
        Arc::new(fs.clone()),
        Arc::new(MockIdGenerator::new()),
        Arc::new(MockClock::new(Utc::now())),
        Arc::new(MockConflictResolver),
    );

    let command = OperationCommand {
        operation_type: OperationType::Copy,
        source_paths: vec![src_path.clone()],
        target_folder_id: Some(folder.id.clone()),
        overwrite_policy: OverwritePolicy::AutoRename,
        target_paths: None,
    };

    // ---- When ----
    let result = use_case.execute(command).await.unwrap();

    // ---- Then ----
    assert_eq!(result.processed_files, 1);
    assert!(fs.exists(&src_path).await.unwrap());       // source still exists
    assert!(fs.exists(&dst_existing).await.unwrap());   // original destination unchanged
    assert!(fs.exists(&dst_renamed).await.unwrap());    // renamed copy created
}