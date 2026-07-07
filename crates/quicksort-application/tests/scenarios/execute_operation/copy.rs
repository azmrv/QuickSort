//! Executable specification for Copy operation.

use std::time::SystemTime;
use std::sync::Arc;

use quicksort_domain::{OperationType, OperationState, DomainEvent};
use quicksort_application::{ExecuteOperation, OperationCommand, OverwritePolicy};
use quicksort_application::use_cases::ExecuteOperationUseCase;
use quicksort_application::errors::UseCaseError;

use crate::common::*;

/// Scenario: Copy a single file to an existing folder.
#[tokio::test]
async fn copy_single_file_to_existing_folder() {
    // ============ GIVEN ============
    let folder = test_folder();
    let config_repo = MockConfigurationRepository::default();
    config_repo.add(folder.clone()).await.unwrap();

    let fs = MockFileSystem::new();
    let src_path = test_file("C:\\Users\\Test\\Downloads\\report.pdf");
    let dst_path = test_file("C:\\Users\\Test\\Documents\\report.pdf");
    fs.create_file(&src_path, 1024);

    let op_repo = MockOperationRepository::default();
    let fixed_time = SystemTime::now();
    let id_gen = StubIdGenerator::new("op-copy-1");

    let use_case = ExecuteOperationUseCase::new(
        Arc::new(config_repo),
        Arc::new(op_repo.clone()),
        Arc::new(fs.clone()),
        Arc::new(id_gen),
        Arc::new(FrozenClock::new(fixed_time)),
        Arc::new(StubConflictResolver),
    );

    let command = OperationCommand {
        operation_type: OperationType::Copy,
        source_paths: vec![src_path.clone()],
        target_folder_id: Some(folder.id.clone()),
        overwrite_policy: OverwritePolicy::Skip,
    };

    // ============ WHEN ============
    let result = use_case.execute(command).await.unwrap();

    // ============ THEN ============
    assert_eq!(result.processed_files, 1);
    assert_eq!(result.bytes_moved, 1024);

    // The file is copied (source still exists)
    assert!(fs.file_exists(&src_path));
    assert!(fs.file_exists(&dst_path));

    let saved_op = op_repo.find_by_id(&result.operation_id).await.unwrap().unwrap();
    assert!(matches!(saved_op.state, OperationState::Completed { .. }));
}