//! Contract tests for ConfigurationRepository.
//!
//! These tests define the expected behavior of any implementation of the port.
//! Every implementation (JsonRepository, InMemoryRepository, etc.) must pass
//! all these tests. This ensures that the application layer can rely on
//! consistent behavior regardless of the underlying storage.
//!
//! Why this is important:
//! - If we ever change the storage (e.g., from JSON to SQLite), the tests
//!   will catch any behavioral differences.
//! - It provides a clear specification for new implementations.

use quicksort_application::ports::ConfigurationRepository;
use quicksort_domain::entities::Folder;
use quicksort_domain::value_objects::{FolderId, FolderName, FolderPath};

/// Runs all contract tests for a given repository implementation.
/// This is a convenience function to call from each implementation's tests.
pub fn run_all_contract_tests<R: ConfigurationRepository>(repo: R) {
    test_save_and_load(repo);
    test_add_folder(repo);
    test_remove_folder(repo);
    test_update_folder(repo);
    test_load_folders_empty(repo);
    test_find_by_id(repo);
    test_find_by_path(repo);
    // Add more tests as needed.
}

/// Test: Save a list of folders and then load it back.
///
/// This verifies the basic round-trip: serialization and deserialization.
/// If this fails, the repository cannot persist data correctly.
fn test_save_and_load<R: ConfigurationRepository>(mut repo: R) {
    let folder = Folder::new(
        FolderId::from_string("1"),
        FolderName::new("Test").unwrap(),
        FolderPath::from_string("C:\\Test"),
    );
    repo.save_folders(&[folder.clone()]).unwrap();
    let loaded = repo.load_folders().unwrap();
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].id, folder.id);
    assert_eq!(loaded[0].name, folder.name);
    assert_eq!(loaded[0].path, folder.path);
}

/// Test: Add a folder to the repository.
///
/// This verifies that the repository can add a new folder without affecting others.
fn test_add_folder<R: ConfigurationRepository>(mut repo: R) {
    let folder = Folder::new(
        FolderId::from_string("2"),
        FolderName::new("Music").unwrap(),
        FolderPath::from_string("C:\\Music"),
    );
    repo.add(folder.clone()).unwrap();
    let loaded = repo.load_folders().unwrap();
    assert!(loaded.iter().any(|f| f.id == folder.id));
}

/// Test: Remove a folder from the repository.
///
/// This verifies that the repository can delete a folder and it's gone.
fn test_remove_folder<R: ConfigurationRepository>(mut repo: R) {
    let folder = Folder::new(
        FolderId::from_string("3"),
        FolderName::new("Videos").unwrap(),
        FolderPath::from_string("C:\\Videos"),
    );
    repo.add(folder.clone()).unwrap();
    repo.remove(&folder.id).unwrap();
    let loaded = repo.load_folders().unwrap();
    assert!(!loaded.iter().any(|f| f.id == folder.id));
}

/// Test: Update a folder's name and path.
///
/// This verifies that the repository can modify an existing folder.
fn test_update_folder<R: ConfigurationRepository>(mut repo: R) {
    let mut folder = Folder::new(
        FolderId::from_string("4"),
        FolderName::new("Old").unwrap(),
        FolderPath::from_string("C:\\Old"),
    );
    repo.add(folder.clone()).unwrap();
    folder.rename(FolderName::new("New").unwrap());
    repo.save_folders(&[folder]).unwrap();
    let loaded = repo.load_folders().unwrap();
    assert!(loaded.iter().any(|f| f.name.as_str() == "New"));
}

/// Test: Loading from an empty repository returns an empty list.
///
/// This verifies that the repository correctly handles the empty state.
fn test_load_folders_empty<R: ConfigurationRepository>(mut repo: R) {
    // For a fresh repository (e.g., new file), load_folders should return empty.
    // We need a way to reset the repository; this is implementation-specific.
    // In tests, we'll use a temporary path or a memory repository.
    let loaded = repo.load_folders().unwrap();
    assert!(loaded.is_empty());
}

/// Test: Find a folder by its ID.
///
/// This verifies that the repository can retrieve a folder by its unique identifier.
fn test_find_by_id<R: ConfigurationRepository>(mut repo: R) {
    let folder = Folder::new(
        FolderId::from_string("5"),
        FolderName::new("Docs").unwrap(),
        FolderPath::from_string("C:\\Docs"),
    );
    repo.add(folder.clone()).unwrap();
    let found = repo.find_by_id(&folder.id).unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().id, folder.id);
}

/// Test: Find a folder by its path.
///
/// This verifies that the repository can retrieve a folder by its full path.
fn test_find_by_path<R: ConfigurationRepository>(mut repo: R) {
    let folder = Folder::new(
        FolderId::from_string("6"),
        FolderName::new("Pics").unwrap(),
        FolderPath::from_string("C:\\Pics"),
    );
    repo.add(folder.clone()).unwrap();
    let found = repo.find_by_path(folder.path.as_str()).unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().id, folder.id);
}

// We can add more contract tests for errors: e.g., what happens when saving fails,
// or when the repository is corrupted. These are implementation-specific,
// but we can define the expected error types.