//! Executable specification for Undo operation.
//!
//! These tests verify that the `UndoOperationUseCase` correctly reverts
//! previously completed operations. Each scenario follows the Given-When-Then
//! structure defined in `SPECIFICATION.md`.

use std::sync::Arc;
use chrono::Utc;

use quicksort_domain::{OperationType, OperationState, WindowsPath, OperationId};
use quicksort_application::{
    ExecuteOperation, OperationCommand, OverwritePolicy, UseCaseError,
    use_cases::{ExecuteOperationUseCase, UndoOperationUseCase},
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
        name: "Documents".to_string(),
        path: wp("C:\\Users\\Test\\Documents"),
        favorite: false,
        order: 0,
        stats: Default::default(),
    }
}

// ============================================================================
// Scenario: Undo a successful Move operation
// ============================================================================

/// Scenario: Undo a previously completed Move operation.
///
/// Given a file was moved from Downloads to Documents,
/// when the Undo operation is executed,
/// then the file should be moved back to its original location.
#[tokio::test]
async fn undo_move_operation() {
    // ---- Given ----
    // Setup configuration with the target folder
    let folder = test_folder();
    let config_repo = MockConfigurationRepository::new();
    config_repo.add(folder.clone()).await.unwrap();

    // Setup file system with a source file
    let src_path = wp("C:\\Users\\Test\\Downloads\\file.txt");
    let dst_path = wp("C:\\Users\\Test\\Documents\\file.txt");

    let fs = MockFileSystem::new();
    fs.add_file(src_path.to_path_buf(), 1024);  // source file exists

    // Shared operation repository (used by both use cases)
    let op_repo = MockOperationRepository::new();

    let id_gen = MockIdGenerator::new();
    let clock = MockClock::new(Utc::now());

    // Step 1: Execute a Move operation
    let execute_use_case = ExecuteOperationUseCase::new(
        Arc::new(config_repo),
        Arc::new(op_repo.clone()),
        Arc::new(fs.clone()),
        Arc::new(id_gen),
        Arc::new(clock.clone()),
        Arc::new(MockConflictResolver),
    );

    let command = OperationCommand {
        operation_type: OperationType::Move,
        source_paths: vec![src_path.clone()],
        target_folder_id: Some(folder.id.clone()),
        overwrite_policy: OverwritePolicy::Skip,
        target_paths: None,
    };

    let result = execute_use_case.execute(command).await.unwrap();
    let op_id = result.operation_id.clone();

    // Verify the move succeeded
    assert!(!fs.exists(&src_path).await.unwrap());
    assert!(fs.exists(&dst_path).await.unwrap());

    // Step 2: Undo the Move operation
    let undo_use_case = UndoOperationUseCase::new(
        Arc::new(op_repo.clone()),
        Arc::new(fs.clone()),
        Arc::new(clock.clone()),
    );

    // ---- When ----
    let undo_result = undo_use_case.undo(op_id.clone()).await.unwrap();

    // ---- Then ----
    // File is back at the original location
    assert!(fs.exists(&src_path).await.unwrap());
    assert!(!fs.exists(&dst_path).await.unwrap());

    // Operation is marked as Undone in the repository
    let saved_op = op_repo.find_by_id(&op_id).await.unwrap().unwrap();
    assert!(matches!(saved_op.state, OperationState::Undone));
}

// ============================================================================
// Scenario: Undo fails for a non-completed operation
// ============================================================================

/// Scenario: Attempt to undo an operation that is not completed.
///
/// Given an operation that is still pending (not yet executed),
/// when an Undo is attempted,
/// then the operation should fail with an `UndoNotPossible` error.
#[tokio::test]
async fn undo_fails_for_non_completed_operation() {
    // ---- Given ----
    let op_repo = MockOperationRepository::new();
    let fs = MockFileSystem::new();
    let clock = MockClock::new(Utc::now());

    // Create a Pending operation (not yet started)
    let pending_op = quicksort_domain::Operation::new_move(
        vec![wp("C:\\src.txt")],
        wp("C:\\dst.txt"),
        Utc::now(),
    );
    op_repo.set_operation(pending_op.clone());

    let undo_use_case = UndoOperationUseCase::new(
        Arc::new(op_repo.clone()),
        Arc::new(fs),
        Arc::new(clock),
    );

    // ---- When ----
    let result = undo_use_case.undo(pending_op.id.clone()).await;

    // ---- Then ----
    assert!(matches!(result, Err(UseCaseError::UndoNotPossible(_))));
}