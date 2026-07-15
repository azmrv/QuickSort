//! Executable specification for Delete operation.
//!
//! These tests verify that the `ExecuteOperationUseCase` correctly performs
//! Delete operations. Each scenario follows the Given-When-Then structure
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
// Scenario: Delete a single file
// ============================================================================

/// Scenario: Delete a single existing file.
///
/// Given a source file that exists on the file system,
/// when a Delete operation is executed,
/// then the file should be removed and the operation recorded.
#[tokio::test]
async fn delete_single_file() {
    // ---- Given ----
    let config_repo = MockConfigurationRepository::new();
    // No folder configuration needed for Delete

    let src_path = wp("C:\\Users\\Test\\Downloads\\temp.txt");

    let fs = MockFileSystem::new();
    fs.add_file(src_path.to_path_buf(), 512);  // file exists with 512 bytes

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
        operation_type: OperationType::Delete,
        source_paths: vec![src_path.clone()],
        target_folder_id: None,           // Delete has no target
        overwrite_policy: OverwritePolicy::Skip,  // not relevant for Delete
        target_paths: None,
    };

    // ---- When ----
    let result = use_case.execute(command).await.unwrap();

    // ---- Then ----
    assert_eq!(result.processed_files, 1);
    // Delete does not report a byte count (the operation aggregate records 0)
    assert_eq!(result.bytes_moved, 0);

    // File must no longer exist
    assert!(!fs.exists(&src_path).await.unwrap());

    // Operation must be persisted
    let saved_op = op_repo.find_by_id(&result.operation_id).await.unwrap().unwrap();
    assert!(matches!(saved_op.state, OperationState::Completed { .. }));
}

// ============================================================================
// Scenario: Delete a non-existent file
// ============================================================================

/// Scenario: Attempt to delete a file that does not exist.
///
/// Given a source path that does not exist on the file system,
/// when a Delete operation is executed,
/// then the operation should fail with a FileNotFound error.
#[tokio::test]
async fn delete_nonexistent_file() {
    // ---- Given ----
    let config_repo = MockConfigurationRepository::new();

    let src_path = wp("C:\\Users\\Test\\Downloads\\missing.txt");

    let fs = MockFileSystem::new();
    // File is NOT added – it does not exist

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
        operation_type: OperationType::Delete,
        source_paths: vec![src_path.clone()],
        target_folder_id: None,
        overwrite_policy: OverwritePolicy::Skip,
        target_paths: None,
    };

    // ---- When ----
    let result = use_case.execute(command).await;

    // ---- Then ----
    assert!(matches!(result, Err(UseCaseError::FileNotFound(_))));
    // No operation should be saved on failure
    assert_eq!(op_repo.count(), 0);
}