//! Executable specification for Conflict resolution.

use std::time::SystemTime;
use std::sync::Arc;

use quicksort_domain::{OperationType, OperationState};
use quicksort_application::{ExecuteOperation, OperationCommand, OverwritePolicy};
use quicksort_application::use_cases::ExecuteOperationUseCase;
use quicksort_application::errors::UseCaseError;

use crate::common::*;

/// Scenario: Conflict with AutoRename policy.
#[tokio::test]
async fn move_with_conflict_auto_rename() {
    // Given: target folder already contains "file.txt"
    let folder = test_folder();
    let config_repo = MockConfigurationRepository::default();
    config_repo.add(folder.clone()).await.unwrap();

    let fs = MockFileSystem::new();
    let src_path = test_file("C:\\Users\\Test\\Downloads\\file.txt");
    let dst_existing = test_file("C:\\Users\\Test\\Documents\\file.txt");
    let dst_renamed = test_file("C:\\Users\\Test\\Documents\\file (1).txt");
    fs.create_file(&src_path, 1024);
    fs.create_file(&dst_existing, 2048); // existing file

    let op_repo = MockOperationRepository::default();
    let fixed_time = SystemTime::now();
    let id_gen = StubIdGenerator::new("op-conflict-1");

    let use_case = ExecuteOperationUseCase::new(
        Arc::new(config_repo),
        Arc::new(op_repo.clone()),
        Arc::new(fs.clone()),
        Arc::new(id_gen),
        Arc::new(FrozenClock::new(fixed_time)),
        Arc::new(StubConflictResolver),
    );

    let command = OperationCommand {
        operation_type: OperationType::Move,
        source_paths: vec![src_path.clone()],
        target_folder_id: Some(folder.id.clone()),
        overwrite_policy: OverwritePolicy::AutoRename,
    };

    // When
    let result = use_case.execute(command).await.unwrap();

    // Then: file is moved to "file (1).txt"
    assert_eq!(result.processed_files, 1);
    assert!(!fs.file_exists(&src_path));
    assert!(fs.file_exists(&dst_existing)); // original stays
    assert!(fs.file_exists(&dst_renamed)); // new file with suffix
}