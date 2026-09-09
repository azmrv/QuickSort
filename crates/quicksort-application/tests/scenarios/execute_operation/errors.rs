//! Executable specification for error cases.
//!
//! These tests verify that the `ExecuteOperationUseCase` correctly handles
//! error conditions, such as missing target folders, invalid commands, and
//! infrastructure failures.
//! Each scenario follows the Given-When-Then structure defined in
//! `SPECIFICATION.md`.

use chrono::Utc;

use quicksort_application::{
    use_cases::ExecuteOperationUseCase, ExecuteOperation, OperationCommand, OverwritePolicy,
    UseCaseError,
};
use quicksort_domain::{AbsolutePath, DuplicateCheckMode, FolderId, OperationType};

use crate::mocks::*;

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
    let src_path = wp("file.txt");
    let fs = MockFileSystem::new();
    fs.add_file(src_path.to_path_buf(), 100); // 100 bytes

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

    // A command targeting a folder that does not exist
    let command = OperationCommand {
        operation_type: OperationType::Move,
        source_paths: vec![src_path],
        target_folder_id: Some(FolderId::new()),
        overwrite_policy: OverwritePolicy::Skip,
        target_paths: None,
        duplicate_check_mode: DuplicateCheckMode::default(),
    };

    // ---- When ----
    let result = use_case.execute(command).await;

    // ---- Then ----
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
        Box::new(op_repo),
        Box::new(config_repo),
        Box::new(fs),
        Box::new(MockIdGenerator::new()),
        Box::new(MockClock::new(Utc::now())),
        Box::new(MockDuplicateDetector),
    );

    let command = OperationCommand {
        operation_type: OperationType::Move,
        source_paths: vec![],
        target_folder_id: Some(FolderId::new()),
        overwrite_policy: OverwritePolicy::Skip,
        target_paths: None,
        duplicate_check_mode: DuplicateCheckMode::default(),
    };

    // ---- When ----
    let result = use_case.execute(command).await;

    // ---- Then ----
    assert!(matches!(result, Err(UseCaseError::InvalidCommand(_))));
}
