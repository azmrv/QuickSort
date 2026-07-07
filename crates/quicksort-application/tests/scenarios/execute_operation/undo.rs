//! Executable specification for Undo operation.

use std::time::SystemTime;
use std::sync::Arc;

use quicksort_domain::{OperationType, OperationState, OperationId};
use quicksort_application::{UndoOperation, OperationCommand, OverwritePolicy};
use quicksort_application::use_cases::{ExecuteOperationUseCase, UndoOperationUseCase};
use quicksort_application::errors::UseCaseError;

use crate::common::*;

/// Scenario: Undo a successful Move operation.
#[tokio::test]
async fn undo_move_operation() {
    // Given: a file was moved
    let config_repo = MockConfigurationRepository::default();
    let folder = test_folder();
    config_repo.add(folder.clone()).await.unwrap();

    let fs = MockFileSystem::new();
    let src_path = test_file("C:\\Users\\Test\\Downloads\\file.txt");
    let dst_path = test_file("C:\\Users\\Test\\Documents\\file.txt");
    fs.create_file(&src_path, 1024);

    let op_repo = MockOperationRepository::default();
    let fixed_time = SystemTime::now();
    let id_gen = StubIdGenerator::new("op-undo-1");

    // Execute the move
    let execute_use_case = ExecuteOperationUseCase::new(
        Arc::new(config_repo.clone()),
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
        overwrite_policy: OverwritePolicy::Skip,
    };

    let result = execute_use_case.execute(command).await.unwrap();
    let op_id = result.operation_id;

    // Now undo
    let undo_use_case = UndoOperationUseCase::new(
        Arc::new(op_repo.clone()),
        Arc::new(fs.clone()),
        Arc::new(FrozenClock::new(SystemTime::now())),
    );

    let undo_result = undo_use_case.undo(op_id.clone()).await.unwrap();

    // Then: file is back to original location
    assert!(fs.file_exists(&src_path));
    assert!(!fs.file_exists(&dst_path));

    let saved_op = op_repo.find_by_id(&op_id).await.unwrap().unwrap();
    assert!(matches!(saved_op.state, OperationState::Undone));
}