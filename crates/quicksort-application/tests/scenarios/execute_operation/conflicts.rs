//! Executable specification for Conflict resolution.
//!
//! These tests verify that the `ExecuteOperationUseCase` correctly handles
//! file conflicts according to the specified `OverwritePolicy`.
//! Each scenario follows the Given-When-Then structure defined in
//! `SPECIFICATION.md`.

use std::sync::Arc;
use chrono::Utc;

use quicksort_domain::{OperationType, OperationState, WindowsPath};
use quicksort_application::{
    ExecuteOperation, OperationCommand, OverwritePolicy, UseCaseError,
    use_cases::ExecuteOperationUseCase,
};
use quicksort_application::ports::outbound::{
    ConfigurationRepository, OperationRepository, FileSystem,
    IdGenerator, Clock, ConflictResolver,
};

use crate::mocks::*;
use crate::scenarios::common::*;

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
// Scenario: Conflict with AutoRename policy
// ============================================================================

/// Scenario: Move with conflict and AutoRename policy.
///
/// Given a target folder that already contains "file.txt",
/// when a Move operation is executed with `OverwritePolicy::AutoRename`,
/// then the source file should be renamed to "file (1).txt" in the target.
#[tokio::test]
async fn move_with_conflict_auto_rename() {
    // ---- Given ----
    let folder = test_folder();
    let config_repo = MockConfigurationRepository::new();
    config_repo.add(folder.clone()).await.unwrap();

    let src_path = wp("C:\\Users\\Test\\Downloads\\file.txt");
    let dst_existing = wp("C:\\Users\\Test\\Documents\\file.txt");
    let dst_renamed = wp("C:\\Users\\Test\\Documents\\file (1).txt");

    let fs = MockFileSystem::new();
    fs.add_file(src_path.to_path_buf(), 1024);   // source file
    fs.add_file(dst_existing.to_path_buf(), 2048); // existing destination

    let op_repo = MockOperationRepository::new();
    let fixed_time = Utc::now();
    let id_gen = MockIdGenerator::new();
    let clock = MockClock::new(fixed_time);
    let conflict_resolver = MockConflictResolver;

    let use_case = ExecuteOperationUseCase::new(
        Arc::new(config_repo),
        Arc::new(op_repo.clone()),
        Arc::new(fs.clone()),
        Arc::new(id_gen),
        Arc::new(clock),
        Arc::new(conflict_resolver),
    );

    let command = OperationCommand {
        operation_type: OperationType::Move,
        source_paths: vec![src_path.clone()],
        target_folder_id: Some(folder.id.clone()),
        overwrite_policy: OverwritePolicy::AutoRename,
        target_paths: None,   // not needed for Move
    };

    // ---- When ----
    let result = use_case.execute(command).await.unwrap();

    // ---- Then ----
    assert_eq!(result.processed_files, 1);
    assert!(matches!(result.state, OperationState::Completed { .. }));

    // Source file should no longer exist
    assert!(!fs.exists(&src_path).await.unwrap());

    // Original destination file should still exist (unchanged)
    assert!(fs.exists(&dst_existing).await.unwrap());

    // A new file with the suffix should have been created
    assert!(fs.exists(&dst_renamed).await.unwrap());

    // The operation should be saved in the repository
    assert_eq!(op_repo.count(), 1);
}

// ============================================================================
// Scenario: Conflict with Skip policy
// ============================================================================

/// Scenario: Move with conflict and Skip policy.
///
/// Given a target folder that already contains "file.txt",
/// when a Move operation is executed with `OverwritePolicy::Skip`,
/// then the operation should fail with a Conflict error.
#[tokio::test]
async fn move_with_conflict_skip() {
    // ---- Given ----
    let folder = test_folder();
    let config_repo = MockConfigurationRepository::new();
    config_repo.add(folder.clone()).await.unwrap();

    let src_path = wp("C:\\Users\\Test\\Downloads\\file.txt");
    let dst_existing = wp("C:\\Users\\Test\\Documents\\file.txt");

    let fs = MockFileSystem::new();
    fs.add_file(src_path.to_path_buf(), 1024);
    fs.add_file(dst_existing.to_path_buf(), 2048); // conflict

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
        operation_type: OperationType::Move,
        source_paths: vec![src_path.clone()],
        target_folder_id: Some(folder.id.clone()),
        overwrite_policy: OverwritePolicy::Skip,
        target_paths: None,
    };

    // ---- When ----
    let result = use_case.execute(command).await;

    // ---- Then ----
    assert!(matches!(result, Err(UseCaseError::Conflict(_))));

    // Source file should still exist (operation failed)
    assert!(fs.exists(&src_path).await.unwrap());
}

// ============================================================================
// Scenario: Conflict with Overwrite policy
// ============================================================================

/// Scenario: Move with conflict and Overwrite policy.
///
/// Given a target folder that already contains "file.txt",
/// when a Move operation is executed with `OverwritePolicy::Overwrite`,
/// then the existing file should be replaced.
#[tokio::test]
async fn move_with_conflict_overwrite() {
    // ---- Given ----
    let folder = test_folder();
    let config_repo = MockConfigurationRepository::new();
    config_repo.add(folder.clone()).await.unwrap();

    let src_path = wp("C:\\Users\\Test\\Downloads\\file.txt");
    let dst_existing = wp("C:\\Users\\Test\\Documents\\file.txt");

    let fs = MockFileSystem::new();
    fs.add_file(src_path.to_path_buf(), 1024);
    fs.add_file(dst_existing.to_path_buf(), 2048);

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
        operation_type: OperationType::Move,
        source_paths: vec![src_path.clone()],
        target_folder_id: Some(folder.id.clone()),
        overwrite_policy: OverwritePolicy::Overwrite,
        target_paths: None,
    };

    // ---- When ----
    let result = use_case.execute(command).await.unwrap();

    // ---- Then ----
    assert_eq!(result.processed_files, 1);
    assert!(!fs.exists(&src_path).await.unwrap()); // source gone
    assert!(fs.exists(&dst_existing).await.unwrap()); // destination exists (overwritten)
}