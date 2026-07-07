//! Executable specification for Rename operation.

use std::time::SystemTime;
use std::sync::Arc;

use quicksort_domain::{OperationType, OperationState};
use quicksort_application::{ExecuteOperation, OperationCommand, OverwritePolicy};
use quicksort_application::use_cases::ExecuteOperationUseCase;

use crate::common::*;

/// Scenario: Rename a file.
#[tokio::test]
async fn rename_file() {
    // Given
    let config_repo = MockConfigurationRepository::default();
    let fs = MockFileSystem::new();
    let old_path = test_file("C:\\Users\\Test\\Downloads\\old_name.txt");
    let new_path = test_file("C:\\Users\\Test\\Downloads\\new_name.txt");
    fs.create_file(&old_path, 1024);

    let op_repo = MockOperationRepository::default();
    let fixed_time = SystemTime::now();
    let id_gen = StubIdGenerator::new("op-rename-1");

    let use_case = ExecuteOperationUseCase::new(
        Arc::new(config_repo),
        Arc::new(op_repo.clone()),
        Arc::new(fs.clone()),
        Arc::new(id_gen),
        Arc::new(FrozenClock::new(fixed_time)),
        Arc::new(StubConflictResolver),
    );

    let command = OperationCommand {
        operation_type: OperationType::Rename,
        source_paths: vec![old_path.clone()],
        target_folder_id: None,
        overwrite_policy: OverwritePolicy::Skip,
        // We need to pass the new name; we can extend OperationCommand to include a new_name field.
        // For simplicity, we'll skip rename for now and implement it later.
    };

    // When
    let result = use_case.execute(command).await.unwrap();

    // Then
    assert_eq!(result.processed_files, 1);
    assert!(!fs.file_exists(&old_path));
    assert!(fs.file_exists(&new_path));
}