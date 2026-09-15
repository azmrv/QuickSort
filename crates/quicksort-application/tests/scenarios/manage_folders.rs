//! Scenario tests for `set_folder_parent` and the `add_folder` parent
//! validation: attach, clear, cycle detection, and depth bounds.

use crate::mocks::MockConfigurationRepository;
use quicksort_application::errors::UseCaseError;
use quicksort_application::ports::inbound::ManageFolders;
use quicksort_application::use_cases::ManageFoldersUseCase;
use quicksort_domain::{AbsolutePath, Folder, FolderId};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use uuid::Uuid;

/// Maps a fixture name to a stable `FolderId` (deterministic across runs).
fn fid(id: &str) -> FolderId {
    let mut hasher = DefaultHasher::new();
    id.hash(&mut hasher);
    FolderId::from_uuid(Uuid::from_u128(hasher.finish() as u128))
}

fn test_path(path: &str) -> AbsolutePath {
    let portable = if cfg!(target_os = "windows") {
        path.to_string()
    } else {
        path.replace('\\', "/").trim_start_matches("C:").to_string()
    };
    AbsolutePath::new(&portable).unwrap()
}

fn node(id: &str, parent: Option<&str>) -> Folder {
    let mut folder = Folder::with_id(
        fid(id),
        format!("Folder {id}"),
        test_path("C:\\QuickSort\\test"),
    );
    if let Some(parent) = parent {
        folder
            .set_parent(Some(fid(parent)))
            .expect("test fixture must avoid self-cycles");
    }
    folder
}

/// Builds a linear chain `f0 -> f1 -> ... -> f(len-1)`.
fn chain(len: usize) -> Vec<Folder> {
    (0..len)
        .map(|i| {
            let parent = if i == 0 {
                None
            } else {
                Some(format!("f{}", i - 1))
            };
            node(&format!("f{i}"), parent.as_deref())
        })
        .collect()
}

fn setup(folders: Vec<Folder>) -> (MockConfigurationRepository, ManageFoldersUseCase) {
    let repo = MockConfigurationRepository::new();
    repo.set_folders(folders);
    let use_case = ManageFoldersUseCase::new(Arc::new(repo.clone()));
    (repo, use_case)
}

fn parent_of(repo: &MockConfigurationRepository, id: &str) -> Option<FolderId> {
    repo.get_folders()
        .iter()
        .find(|f| f.id == fid(id))
        .expect("folder must exist")
        .parent_id
}

#[tokio::test]
async fn set_parent_attaches_folder() {
    let (repo, use_case) = setup(vec![node("a", None), node("b", None)]);
    use_case
        .set_folder_parent(fid("b"), Some(fid("a")))
        .await
        .unwrap();
    assert_eq!(parent_of(&repo, "b"), Some(fid("a")));
}

#[tokio::test]
async fn set_parent_clears_parent() {
    let (repo, use_case) = setup(vec![node("a", None), node("b", Some("a"))]);
    use_case.set_folder_parent(fid("b"), None).await.unwrap();
    assert_eq!(parent_of(&repo, "b"), None);
}

#[tokio::test]
async fn set_parent_unknown_folder_fails() {
    let (repo, use_case) = setup(vec![node("a", None)]);
    let err = use_case
        .set_folder_parent(fid("missing"), Some(fid("a")))
        .await
        .unwrap_err();
    assert!(matches!(err, UseCaseError::FolderNotFound(_)));
    assert_eq!(parent_of(&repo, "a"), None);
}

#[tokio::test]
async fn set_parent_unknown_parent_fails() {
    let (repo, use_case) = setup(vec![node("a", None)]);
    let err = use_case
        .set_folder_parent(fid("a"), Some(fid("missing")))
        .await
        .unwrap_err();
    assert!(matches!(err, UseCaseError::FolderNotFound(_)));
    assert_eq!(parent_of(&repo, "a"), None);
}

#[tokio::test]
async fn set_parent_self_cycle_fails() {
    let (_, use_case) = setup(vec![node("a", None)]);
    let err = use_case
        .set_folder_parent(fid("a"), Some(fid("a")))
        .await
        .unwrap_err();
    assert!(err.to_string().contains("cycle"));
}

#[tokio::test]
async fn set_parent_indirect_cycle_fails() {
    let (repo, use_case) = setup(vec![
        node("a", None),
        node("b", Some("a")),
        node("c", Some("b")),
    ]);
    let err = use_case
        .set_folder_parent(fid("a"), Some(fid("c")))
        .await
        .unwrap_err();
    assert!(err.to_string().contains("cycle"));
    assert_eq!(parent_of(&repo, "a"), None);
}

#[tokio::test]
async fn set_parent_preserves_siblings() {
    let (repo, use_case) = setup(vec![node("a", None), node("b", None), node("c", None)]);
    use_case
        .set_folder_parent(fid("b"), Some(fid("a")))
        .await
        .unwrap();
    let folders = repo.get_folders();
    assert_eq!(
        folders.iter().find(|f| f.id == fid("c")).unwrap().parent_id,
        None,
        "reordering one folder must not affect unrelated folders"
    );
}

#[tokio::test]
async fn set_parent_allows_maximum_depth() {
    let mut folders = chain(9);
    folders.push(node("x", None));
    let (repo, use_case) = setup(folders);
    use_case
        .set_folder_parent(fid("x"), Some(fid("f8")))
        .await
        .unwrap();
    assert_eq!(parent_of(&repo, "x"), Some(fid("f8")));
}

#[tokio::test]
async fn set_parent_rejects_depth_exceeding_move() {
    // `f9` sits at level 10 (root = level 1); a leaf below it would be 11.
    let mut folders = chain(10);
    folders.push(node("x", None));
    let (repo, use_case) = setup(folders);
    let err = use_case
        .set_folder_parent(fid("x"), Some(fid("f9")))
        .await
        .unwrap_err();
    assert!(err.to_string().contains("depth"));
    assert_eq!(parent_of(&repo, "x"), None);
}

#[tokio::test]
async fn set_parent_rejects_deep_subtree_move() {
    // `f8` is at level 9; moving `x` (which has child `y`) under it would
    // push `y` to level 11.
    let mut folders = chain(9);
    folders.push(node("x", None));
    folders.push(node("y", Some("x")));
    let (repo, use_case) = setup(folders);
    let err = use_case
        .set_folder_parent(fid("x"), Some(fid("f8")))
        .await
        .unwrap_err();
    assert!(err.to_string().contains("depth"));
    assert_eq!(parent_of(&repo, "x"), None);
}

#[tokio::test]
async fn add_folder_with_existing_parent_ok() {
    let (repo, use_case) = setup(vec![node("a", None)]);
    use_case.add_folder(node("b", Some("a"))).await.unwrap();
    assert_eq!(parent_of(&repo, "b"), Some(fid("a")));
}

#[tokio::test]
async fn add_folder_unknown_parent_fails() {
    let (repo, use_case) = setup(vec![node("a", None)]);
    let err = use_case
        .add_folder(node("b", Some("missing")))
        .await
        .unwrap_err();
    assert!(matches!(err, UseCaseError::FolderNotFound(_)));
    assert!(repo.get_folders().iter().all(|f| f.id != fid("b")));
}

#[tokio::test]
async fn add_folder_rejects_too_deep_parent() {
    let (repo, use_case) = setup(chain(10));
    let err = use_case
        .add_folder(node("x", Some("f9")))
        .await
        .unwrap_err();
    assert!(err.to_string().contains("depth"));
    assert!(repo.get_folders().iter().all(|f| f.id != fid("x")));
}

#[tokio::test]
async fn add_folder_without_parent_ok() {
    let (repo, use_case) = setup(Vec::new());
    use_case.add_folder(node("a", None)).await.unwrap();
    assert_eq!(parent_of(&repo, "a"), None);
}
