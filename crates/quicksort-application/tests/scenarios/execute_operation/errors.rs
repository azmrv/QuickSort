//! Executable specification for error cases.

use std::time::SystemTime;
use std::sync::Arc;

use quicksort_domain::{OperationType, OperationState};
use quicksort_application::{ExecuteOperation, OperationCommand, OverwritePolicy};
use quicksort_application::use_cases::ExecuteOperationUseCase;
use quicksort_application::errors::UseCaseError;

use crate::common::*;

/// Scenario: Target folder does not exist.
#[tokio::test]
async fn move_target_folder_not_found() {
    let config_repo = MockConfigurationRepository::default();
    let fs = MockFileSystem::new();
    let src_path = test_file("C:\\file.txt");
    fs.create_file(&src_path, 100);

    let op_repo = MockOperationRepository::default();
    let id_gen = StubIdGenerator::new("op-err-1");

    let use_case = ExecuteOperationUseCase::new(
        Arc::new(config_repo),
        Arc::new(op_repo),
        Arc::new(fs),
        Arc::new(id_gen),
        Arc::new(FrozenClock::new(SystemTime::now())),
        Arc::new(StubConflictResolver),
    );

    let command = OperationCommand {
        operation_type: OperationType::Move,
        source_paths: vec![src_path],
        target_folder_id: Some(FolderId::from_string("non-existent")),
        overwrite_policy: OverwritePolicy::Skip,
    };

    let result = use_case.execute(command).await;

    assert!(matches!(result, Err(UseCaseError::FolderNotFound(_))));
}