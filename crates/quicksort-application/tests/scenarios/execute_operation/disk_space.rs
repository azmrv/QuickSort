//! Executable specification for the disk space pre-flight check.
//!
//! These tests verify that `ExecuteOperationUseCase` fails with an
//! `InsufficientDiskSpace` error BEFORE any file is moved or copied when
//! the target volume lacks the space the operation needs (re-QA 12.09.2026
//! D3: a 2.6 GB Move failed mid-operation with "Недостаточно места на
//! диске").
//! Each scenario follows the Given-When-Then structure defined in
//! `SPECIFICATION.md`.

use chrono::Utc;

use quicksort_application::{
    use_cases::ExecuteOperationUseCase, ExecuteOperation, OperationCommand, OverwritePolicy,
    UseCaseError,
};
use quicksort_domain::{AbsolutePath, DuplicateCheckMode, OperationType};

use crate::mocks::*;
use crate::scenarios::test_folder;

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

fn use_case(
    config_repo: MockConfigurationRepository,
    fs: MockFileSystem,
) -> (ExecuteOperationUseCase, MockOperationRepository) {
    let op_repo = MockOperationRepository::new();
    let use_case = ExecuteOperationUseCase::new(
        Box::new(op_repo.clone()),
        Box::new(config_repo),
        Box::new(fs),
        Box::new(MockIdGenerator::new()),
        Box::new(MockClock::new(Utc::now())),
        Box::new(MockDuplicateDetector),
    );
    (use_case, op_repo)
}

fn command(
    op_type: OperationType,
    src: &AbsolutePath,
    folder_id: quicksort_domain::FolderId,
) -> OperationCommand {
    OperationCommand {
        operation_type: op_type,
        source_paths: vec![src.clone()],
        target_folder_id: Some(folder_id),
        overwrite_policy: OverwritePolicy::Skip,
        target_paths: None,
        duplicate_check_mode: DuplicateCheckMode::default(),
        ..OperationCommand::default()
    }
}

// ============================================================================
// Scenario: Copy fails before touching files when the target lacks space
// ============================================================================

/// Scenario: Copy with insufficient space on the target volume.
///
/// Given a source file and a target folder on a volume with less free
/// space than the source size,
/// when a Copy operation is executed,
/// then it should fail with `InsufficientDiskSpace`, the destination must
/// NOT be created, and the operation is persisted as Failed.
#[tokio::test]
async fn copy_fails_when_target_volume_lacks_space() {
    // ---- Given ----
    let folder = test_folder();
    let config_repo = MockConfigurationRepository::new();
    config_repo.add(folder.clone()).await.unwrap();

    let src_path = wp("Downloads/big_file.bin");
    let dst_path = wp("Documents/big_file.bin");

    let fs = MockFileSystem::new();
    fs.add_file(src_path.to_path_buf(), 1024); // source is 1024 bytes
    fs.set_available_space(512); // target volume has only 512 bytes free

    let (use_case, op_repo) = use_case(config_repo, fs.clone());
    let cmd = command(OperationType::Copy, &src_path, folder.id.clone());

    // ---- When ----
    let result = use_case.execute(cmd).await;

    // ---- Then ----
    assert!(matches!(
        result,
        Err(UseCaseError::InsufficientDiskSpace {
            need: 1024,
            available: 512
        })
    ));
    // Nothing was copied: destination does not exist.
    assert!(!fs.exists(&dst_path).await.unwrap());
    // The failed operation is persisted for the history/undo UI.
    let saved_ops = op_repo.load_all().await.unwrap();
    assert_eq!(saved_ops.len(), 1);
    assert!(matches!(
        saved_ops[0].state,
        quicksort_domain::OperationState::Failed { .. }
    ));
}

// ============================================================================
// Scenario: Same-volume Move is a rename and needs no space
// ============================================================================

/// Scenario: Same-volume Move succeeds regardless of free space.
///
/// Given a source file on the same volume as the target folder,
/// when a Move operation is executed with very little free space,
/// then it should succeed because a same-volume move is a metadata rename.
#[tokio::test]
async fn same_volume_move_ignores_free_space() {
    // ---- Given ----
    let folder = test_folder();
    let config_repo = MockConfigurationRepository::new();
    config_repo.add(folder.clone()).await.unwrap();

    let src_path = wp("Downloads/big_file.bin");
    let dst_path = wp("Documents/big_file.bin");

    let fs = MockFileSystem::new();
    fs.add_file(src_path.to_path_buf(), 1024);
    // Same volume by default (mock default); even 1 byte free must not block.
    fs.set_available_space(1);

    let (use_case, op_repo) = use_case(config_repo, fs.clone());
    let cmd = command(OperationType::Move, &src_path, folder.id.clone());

    // ---- When ----
    let result = use_case.execute(cmd).await;

    // ---- Then ----
    assert!(
        result.is_ok(),
        "same-volume move must not require free space"
    );
    assert!(fs.exists(&dst_path).await.unwrap());
    assert_eq!(op_repo.count(), 1);
}

// ============================================================================
// Scenario: Cross-volume Move needs space for the copy phase
// ============================================================================

/// Scenario: Cross-volume Move fails when the target volume lacks space.
///
/// Given a source file on a different volume than the target folder,
/// when a Move operation is executed with less free space than the
/// source size,
/// then it should fail with `InsufficientDiskSpace` because a cross-volume
/// move copies first and only then deletes the source, and the operation
/// is persisted as Failed.
#[tokio::test]
async fn cross_volume_move_fails_when_target_lacks_space() {
    // ---- Given ----
    let folder = test_folder();
    let config_repo = MockConfigurationRepository::new();
    config_repo.add(folder.clone()).await.unwrap();

    let src_path = wp("Downloads/big_file.bin");
    let dst_path = wp("Documents/big_file.bin");

    let fs = MockFileSystem::new();
    fs.add_file(src_path.to_path_buf(), 1024);
    fs.set_same_volume(false); // source is on another volume
    fs.set_available_space(512);

    let (use_case, op_repo) = use_case(config_repo, fs.clone());
    let cmd = command(OperationType::Move, &src_path, folder.id.clone());

    // ---- When ----
    let result = use_case.execute(cmd).await;

    // ---- Then ----
    assert!(matches!(
        result,
        Err(UseCaseError::InsufficientDiskSpace {
            need: 1024,
            available: 512
        })
    ));
    // Nothing moved: destination does not exist.
    assert!(!fs.exists(&dst_path).await.unwrap());
    // The failed operation is persisted for the history/undo UI.
    let saved_ops = op_repo.load_all().await.unwrap();
    assert_eq!(saved_ops.len(), 1);
    assert!(matches!(
        saved_ops[0].state,
        quicksort_domain::OperationState::Failed { .. }
    ));
}
