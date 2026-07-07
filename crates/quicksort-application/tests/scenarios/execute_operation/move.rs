//! Executable specification for ExecuteOperationUseCase - Move scenarios.

use std::time::SystemTime;
use std::sync::Arc;

use quicksort_domain::{Folder, FolderId, WindowsPath, OperationType, OperationState, DomainEvent};
use quicksort_application::{ExecuteOperation, OperationCommand, OperationResult, OverwritePolicy};
use quicksort_application::use_cases::ExecuteOperationUseCase;
use quicksort_application::errors::UseCaseError;

use crate::mocks::*;

/// Scenario: Move a single file to an existing folder.
#[tokio::test]
async fn move_single_file_to_existing_folder() {
    // ============ GIVEN ============
    // A folder "Documents" exists in configuration
    let folder = test_folder();
    let config_repo = MockConfigurationRepository::default();
    config_repo.add(folder.clone()).await.unwrap();

    // A file "report.pdf" exists in Downloads
    let fs = MockFileSystem::new();
    let src_path = test_file("C:\\Users\\Test\\Downloads\\report.pdf");
    let dst_path = test_file("C:\\Users\\Test\\Documents\\report.pdf");
    fs.create_file(&src_path, 1024);

    // Operation repository for tracking history
    let op_repo = MockOperationRepository::default();

    // Fixed time and ID for deterministic testing
    let fixed_time = SystemTime::now();
    let id_gen = StubIdGenerator::new("op-123");

    // Conflict resolver that auto-renames
    let conflict_resolver = StubConflictResolver;

    // ============ WHEN ============
    let use_case = ExecuteOperationUseCase::new(
        Arc::new(config_repo),
        Arc::new(op_repo.clone()),
        Arc::new(fs.clone()),
        Arc::new(id_gen),
        Arc::new(FrozenClock::new(fixed_time)),
        Arc::new(conflict_resolver),
    );

    let command = OperationCommand {
        operation_type: OperationType::Move,
        source_paths: vec![src_path.clone()],
        target_folder_id: Some(folder.id.clone()),
        overwrite_policy: OverwritePolicy::Skip,
    };

    let result = use_case.execute(command).await.unwrap();

    // ============ THEN ============
    // The operation was successful
    assert_eq!(result.operation_id.as_str(), "op-123");
    assert_eq!(result.processed_files, 1);
    assert_eq!(result.bytes_moved, 1024);

    // The file was moved (no longer at source, exists at destination)
    assert!(!fs.file_exists(&src_path));
    assert!(fs.file_exists(&dst_path));

    // The operation is saved in the repository as Completed
    let saved_op = op_repo.find_by_id(&result.operation_id).await.unwrap().unwrap();
    assert!(matches!(saved_op.state, OperationState::Completed { processed_files: 1, bytes_moved: 1024 }));
    assert_eq!(saved_op.updated_at, fixed_time);
}

/// Scenario: Move fails when source file is missing.
#[tokio::test]
async fn move_fails_when_source_missing() {
    // Given: empty file system (no files)
    let fs = MockFileSystem::new();
    let folder = test_folder();
    let config_repo = MockConfigurationRepository::default();
    config_repo.add(folder.clone()).await.unwrap();

    let src_path = test_file("C:\\missing.txt");
    let dst_path = test_file("C:\\Users\\Test\\Documents\\missing.txt");

    let op_repo = MockOperationRepository::default();
    let id_gen = StubIdGenerator::new("op-fail");

    let use_case = ExecuteOperationUseCase::new(
        Arc::new(config_repo),
        Arc::new(op_repo.clone()),
        Arc::new(fs),
        Arc::new(id_gen),
        Arc::new(FrozenClock::new(SystemTime::now())),
        Arc::new(StubConflictResolver),
    );

    let command = OperationCommand {
        operation_type: OperationType::Move,
        source_paths: vec![src_path],
        target_folder_id: Some(folder.id),
        overwrite_policy: OverwritePolicy::Skip,
    };

    // When: execute the command
    let result = use_case.execute(command).await;

    // Then: we get a FileNotFound error
    assert!(matches!(result, Err(UseCaseError::FileNotFound(_))));

    // And: the operation is saved as Failed
    let saved_op = op_repo.find_by_id(&OperationId::from_string("op-fail")).await.unwrap().unwrap();
    assert!(matches!(saved_op.state, OperationState::Failed { .. }));
}