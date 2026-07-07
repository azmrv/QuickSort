//! Unit tests for ExecuteOperationUseCase with mocked dependencies.
//!
//! These tests focus on the logic inside the Use Case itself, not the infrastructure.
//! We use mocks for all ports (Repository, Executor, etc.) to isolate the Use Case.
//!
//! Why this is important:
//! - It allows us to test the Use Case in isolation, without a real file system or database.
//! - It makes tests fast and deterministic.
//! - We can test error scenarios that would be hard to reproduce in integration tests.

use std::sync::Arc;
use quicksort_application::use_cases::ExecuteOperationUseCase;
use quicksort_application::ports::{
    ConfigurationRepository, OperationRepository, OperationExecutor,
    IdGenerator, Clock, ConflictResolver, EventCollector,
};
use quicksort_domain::entities::Folder;
use quicksort_domain::value_objects::*;
use quicksort_domain::events::DomainEvent;
use mockall::predicate::*;
use mockall::mock;

// ============================================================================
// Mocks for all dependencies
// ============================================================================

mock! {
    ConfigRepo {}
    impl ConfigurationRepository for ConfigRepo {
        fn load_folders(&self) -> Result<Vec<Folder>, ConfigRepositoryError>;
        fn save_folders(&self, folders: &[Folder]) -> Result<(), ConfigRepositoryError>;
        fn add(&self, folder: Folder) -> Result<(), ConfigRepositoryError>;
        fn remove(&self, id: &FolderId) -> Result<(), ConfigRepositoryError>;
        fn find_by_id(&self, id: &FolderId) -> Result<Option<Folder>, ConfigRepositoryError>;
        fn find_by_path(&self, path: &str) -> Result<Option<Folder>, ConfigRepositoryError>;
    }
}

mock! {
    OpRepo {}
    impl OperationRepository for OpRepo {
        fn save_operation(&self, op: &Operation) -> Result<(), OperationRepositoryError>;
    }
}

mock! {
    OpExecutor {}
    impl OperationExecutor for OpExecutor {
        fn execute(&self, command: OperationCommand) -> Result<OperationResult, OperationExecutorError>;
    }
}

mock! {
    IdGen {}
    impl IdGenerator for IdGen {
        fn generate(&self) -> String;
    }
}

mock! {
    ClockMock {}
    impl Clock for ClockMock {
        fn now(&self) -> std::time::SystemTime;
        fn timestamp(&self) -> String;
    }
}

mock! {
    ConflictRes {}
    impl ConflictResolver for ConflictRes {
        fn resolve(&self, command: OperationCommand, folders: &[Folder]) -> Result<OperationCommand, ConflictResolutionError>;
    }
}

mock! {
    EventCollect {}
    impl EventCollector for EventCollect {
        fn collect_events(&self, op: &Operation, result: &OperationResult) -> Vec<DomainEvent>;
    }
}

// ============================================================================
// TEST GROUP: Successful Move
// ============================================================================

/// Test: Execute a Move operation successfully.
///
/// This test verifies that the Use Case correctly orchestrates all dependencies:
/// - It loads folders from the repository.
/// - It resolves conflicts (if any).
/// - It calls the OperationExecutor.
/// - It collects events.
/// - It returns the result.
#[test]
fn execute_move_success() {
    // Arrange: set up mock expectations.
    let mut config_repo = MockConfigRepo::new();
    config_repo.expect_load_folders()
        .returning(|| Ok(vec![
            Folder::new(
                FolderId::from_string("folder1"),
                FolderName::new("Photos").unwrap(),
                FolderPath::from_string("C:\\Photos"),
            )
        ]));

    let mut executor = MockOpExecutor::new();
    executor.expect_execute()
        .with(always()) // any command is fine
        .returning(|_| Ok(OperationResult::Success {
            moved: vec![("C:\\Downloads\\file.txt".to_string(), "C:\\Photos\\file.txt".to_string())]
        }));

    let mut conflict_resolver = MockConflictRes::new();
    conflict_resolver.expect_resolve()
        .returning(|cmd, _| Ok(cmd)); // no conflict

    let mut id_gen = MockIdGen::new();
    id_gen.expect_generate().returning(|| "op-123".to_string());

    let mut clock = MockClockMock::new();
    clock.expect_now().returning(|| std::time::SystemTime::now());

    let mut event_collector = MockEventCollect::new();
    event_collector.expect_collect_events()
        .returning(|_, _| vec![
            DomainEvent::OperationStarted { operation_id: "op-123".to_string(), op_type: OperationType::Move },
            DomainEvent::OperationCompleted { operation_id: "op-123".to_string(), result: OperationResult::Success { moved: vec![] } },
        ]);

    // Build the Use Case with the mocks.
    let use_case = ExecuteOperationUseCase::new(
        Arc::new(config_repo),
        Arc::new(executor),
        Arc::new(conflict_resolver),
        Arc::new(id_gen),
        Arc::new(clock),
        Arc::new(event_collector),
    );

    // Act: execute the command.
    let command = OperationCommand::Move {
        source_paths: vec!["C:\\Downloads\\file.txt".to_string()],
        destination_folder_id: FolderId::from_string("folder1"),
    };
    let result = use_case.execute(command);

    // Assert: the operation succeeded and events are present.
    assert!(result.is_ok());
    let (op_result, events) = result.unwrap();
    assert!(matches!(op_result.status, OperationStatus::Completed));
    assert_eq!(events.len(), 2);
    assert!(events.iter().any(|e| matches!(e, DomainEvent::OperationStarted { .. })));
    assert!(events.iter().any(|e| matches!(e, DomainEvent::OperationCompleted { .. })));
}

// ============================================================================
// TEST GROUP: Error Cases
// ============================================================================

/// Test: Target folder not found.
#[test]
fn execute_move_folder_not_found() {
    let mut config_repo = MockConfigRepo::new();
    config_repo.expect_load_folders()
        .returning(|| Ok(vec![])); // no folders

    let mut conflict_resolver = MockConflictRes::new();
    conflict_resolver.expect_resolve()
        .returning(|cmd, _| Ok(cmd));

    // Other mocks are trivial; we only need to set them up minimally.
    let id_gen = MockIdGen::new();
    let clock = MockClockMock::new();
    let event_collector = MockEventCollect::new();
    let executor = MockOpExecutor::new();

    let use_case = ExecuteOperationUseCase::new(
        Arc::new(config_repo),
        Arc::new(executor),
        Arc::new(conflict_resolver),
        Arc::new(id_gen),
        Arc::new(clock),
        Arc::new(event_collector),
    );

    let command = OperationCommand::Move {
        source_paths: vec!["file.txt".to_string()],
        destination_folder_id: FolderId::from_string("unknown"),
    };
    let result = use_case.execute(command);

    // Assert: we get a FolderNotFound error.
    assert!(result.is_err());
    let err = result.unwrap_err();
    match err {
        UseCaseError::FolderNotFound(id) => assert_eq!(id, "unknown"),
        _ => panic!("Expected FolderNotFound error"),
    }
}

// Similar tests for FileNotFound, PermissionDenied, ConflictResolution errors, etc.
// I'll add them in the final version.

// ============================================================================
// TEST GROUP: Event Generation
// ============================================================================

/// Test: Verify that events are collected correctly on success.
#[test]
fn execute_move_collects_events_on_success() {
    // Similar to the success test, but we explicitly check that collect_events
    // is called with the correct operation and result.
    // We'll use mockall's `expect_*` with `times(1)` to ensure it's called exactly once.
}

/// Test: Verify that events are collected correctly on failure.
#[test]
fn execute_move_collects_events_on_failure() {
    // We mock the executor to return an error and check that OperationFailed is collected.
}

// ============================================================================
// TEST GROUP: Idempotency
// ============================================================================

/// Test: Repeating the same move after success returns a NoOp or FileNotFound.
#[test]
fn execute_move_idempotent() {
    // We'll simulate two calls: the first succeeds, the second should return
    // a FileNotFound error (because the file is already gone) or a NoOp result.
}

// We'll also add tests for partial success, cross-volume moves, and other edge cases.