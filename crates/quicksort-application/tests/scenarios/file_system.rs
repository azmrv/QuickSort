//! Scenario tests for the `FileSystem` port extensions:
//! `generate_unique_path` and `folder_metadata`, driven through the
//! in-memory `MockFileSystem`.

use crate::mocks::MockFileSystem;
use quicksort_application::ports::outbound::FileSystem;
use quicksort_domain::AbsolutePath;
use std::path::PathBuf;

/// Builds an absolute path the mock can key on.
fn abs(path: PathBuf) -> AbsolutePath {
    AbsolutePath::from(path)
}

#[tokio::test]
async fn generate_unique_path_returns_same_path_when_free() {
    let fs = MockFileSystem::new();
    let path = abs(PathBuf::from(format!(
        "{root}Documents\\report.txt",
        root = if cfg!(target_os = "windows") {
            "C:\\"
        } else {
            "/"
        }
    )));

    let unique = fs.generate_unique_path(&path).await.unwrap();
    assert_eq!(unique, path);
}

#[tokio::test]
async fn generate_unique_path_adds_numbered_suffix() {
    let fs = MockFileSystem::new();
    let base = if cfg!(target_os = "windows") {
        "C:\\Documents\\report.txt"
    } else {
        "/Documents/report.txt"
    };
    let path = abs(PathBuf::from(base));

    fs.add_file(PathBuf::from(base), 10);
    fs.add_file(
        PathBuf::from(if cfg!(target_os = "windows") {
            "C:\\Documents\\report (1).txt"
        } else {
            "/Documents/report (1).txt"
        }),
        20,
    );

    let unique = fs.generate_unique_path(&path).await.unwrap();
    assert_eq!(
        unique.to_string_lossy(),
        if cfg!(target_os = "windows") {
            "C:\\Documents\\report (2).txt"
        } else {
            "/Documents/report (2).txt"
        }
    );
}

#[tokio::test]
async fn folder_metadata_reports_existing_file() {
    let fs = MockFileSystem::new();
    let base = if cfg!(target_os = "windows") {
        "C:\\Documents\\report.txt"
    } else {
        "/Documents/report.txt"
    };
    fs.add_file(PathBuf::from(base), 4096);

    let meta = fs.folder_metadata(&abs(PathBuf::from(base))).await.unwrap();
    assert!(meta.exists);
    assert!(!meta.is_dir);
    assert_eq!(meta.total_size, 4096);
    assert_eq!(meta.item_count, 1);
}

#[tokio::test]
async fn folder_metadata_reports_missing_path() {
    let fs = MockFileSystem::new();
    let base = if cfg!(target_os = "windows") {
        "C:\\Documents\\missing.txt"
    } else {
        "/Documents/missing.txt"
    };

    let meta = fs.folder_metadata(&abs(PathBuf::from(base))).await.unwrap();
    assert!(!meta.exists);
    assert!(!meta.is_dir);
    assert_eq!(meta.total_size, 0);
    assert_eq!(meta.item_count, 0);
}
