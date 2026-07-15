//! Executable specification for error cases.
//!
//! These tests verify that the `ExecuteOperationUseCase` correctly handles
//! error conditions, such as missing target folders, invalid commands, and
//! infrastructure failures.
//! Each scenario follows the Given-When-Then structure defined in
//! `SPECIFICATION.md`.

use std::sync::Arc;
use chrono::Utc;

use quicksort_domain::{OperationType, OperationState, WindowsPath, FolderId};
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
// Scenario: Target folder not found
// ============================================================================

/// Scenario: Move operation with a non-existent target folder.
///
/// Given a source file that exists and a target folder ID that does not
/// correspond to any configured folder,
/// when a Move operation is executed,
/// then the operation should fail with a `FolderNotFound` error.
#[tokio::test]
async fn move_target_folder_not_found() {
    // ---- Given ----
    // An empty configuration repository – no folders configured
    let config_repo = MockConfigurationRepository::new();

    // A file system with a single source file
    let src_path = wp("C:\\file.txt");
    let fs = MockFileSystem::new();
    fs.add_file(src_path.to_path_buf(), 100);  // 100 bytes

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

    // A command targeting a folder that does not exist
    // OLD: target_folder_id: Some(FolderId::from_string("non-existent"))
    // NEW: FolderId::from_string is not a constructor; we create a FolderId
    // directly using a test helper or the newtype's constructor.
    let command = OperationCommand {
        operation_type: OperationType::Move,
        source_paths: vec![src_path],
        target_folder_id: Some("non-existent".to_string()),
        overwrite_policy: OverwritePolicy::Skip,
        target_paths: None,   // required field
    };

    // ---- When ----
    let result = use_case.execute(command).await;

    // ---- Then ----
    // OLD: assert!(matches!(result, Err(UseCaseError::FolderNotFound(_))));
    // The error variant may be different depending on how the use case
    // resolves the folder ID. With our current implementation,
    // OperationCommand::target_folder_id is a plain String, and the
    // use case calls config_repo.load_all().await and then searches
    // for a matching folder. If none is found, it returns
    // UseCaseError::FolderNotFound.
    assert!(matches!(result, Err(UseCaseError::FolderNotFound(_))));
}

// ============================================================================
// Scenario: No source paths provided
// ============================================================================

/// Scenario: Operation command with an empty source path list.
///
/// Given a command that has no source paths,
/// when it is passed to the use case,
/// then the operation should fail with an `InvalidCommand` error.
#[tokio::test]
async fn empty_source_paths() {
    // ---- Given ----
    let config_repo = MockConfigurationRepository::new();
    let fs = MockFileSystem::new();
    let op_repo = MockOperationRepository::new();

    let use_case = ExecuteOperationUseCase::new(
        Arc::new(config_repo),
        Arc::new(op_repo),
        Arc::new(fs),
        Arc::new(MockIdGenerator::new()),
        Arc::new(MockClock::new(Utc::now())),
        Arc::new(MockConflictResolver),
    );

    let command = OperationCommand {
        operation_type: OperationType::Move,
        source_paths: vec![],          // empty source list
        target_folder_id: Some("folder-1".to_string()),
        overwrite_policy: OverwritePolicy::Skip,
        target_paths: None,
    };

    // ---- When ----
    let result = use_case.execute(command).await;

    // ---- Then ----
    // The use case should detect the empty source list and return
    // an InvalidCommand error. This validation is currently performed
    // in the pipeline (pipeline::validate_command). If the use case
    // does not use the pipeline, this test will fail, indicating that
    // validation should be added.
    assert!(matches!(result, Err(UseCaseError::InvalidCommand(_))));
}