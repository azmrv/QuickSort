//! Executable specification for Delete operation.

use std::time::SystemTime;
use std::sync::Arc;

use quicksort_domain::{OperationType, OperationState};
use quicksort_application::{ExecuteOperation, OperationCommand, OverwritePolicy};
use quicksort_application::use_cases::ExecuteOperationUseCase;
use quicksort_application::errors::UseCaseError;

use crate::common::*;

/// Scenario: Delete a single file.
#[tokio::test]
async fn delete_single_file() {
    // Given
    let config_repo = MockConfigurationRepository::default();
    let fs = MockFileSystem::new();
    let path = test_file("C:\\Users\\Test\\Downloads\\temp.txt");
    fs.create_file(&path, 512);

    let op_repo = MockOperationRepository::default();
    let fixed_time = SystemTime::now();
    let id_gen = StubIdGenerator::new("op-del-1");

    let use_case = ExecuteOperationUseCase::new(
        Arc::new(config_repo),
        Arc::new(op_repo.clone()),
        Arc::new(fs.clone()),
        Arc::new(id_gen),
        Arc::new(FrozenClock::new(fixed_time)),
        Arc::new(StubConflictResolver),
    );

    let command = OperationCommand {
        operation_type: OperationType::Delete,
        source_paths: vec![path.clone()],
        target_folder_id: None,
        overwrite_policy: OverwritePolicy::Skip,
    };

    // When
    let result = use_case.execute(command).await.unwrap();

    // Then
    assert_eq!(result.processed_files, 1);
    assert_eq!(result.bytes_moved, 0); // Delete doesn't move bytes, we can set to 0

    // File is removed
    assert!(!fs.file_exists(&path));

    let saved_op = op_repo.find_by_id(&result.operation_id).await.unwrap().unwrap();
    assert!(matches!(saved_op.state, OperationState::Completed { .. }));
}