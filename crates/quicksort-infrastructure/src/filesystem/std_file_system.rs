//! Standard implementation of the FileSystem port using tokio::fs.

use std::future::Future;
use std::pin::Pin;

use async_trait::async_trait;
use tokio::fs as tokio_fs;

use quicksort_application::errors::UseCaseError;
use quicksort_application::ports::outbound::FileSystem;
use quicksort_domain::AbsolutePath;

/// Real file system implementation backed by tokio.
pub struct StdFileSystem;

impl StdFileSystem {
    /// Creates a new StdFileSystem.
    pub fn new() -> Self {
        Self
    }
}

impl Default for StdFileSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl StdFileSystem {
    /// Recursively computes the total size (in bytes) of all regular files
    /// under `path`. Symlinks and junctions are not followed.
    ///
    /// Used by the tree move flow to measure sizes before the source
    /// disappears (a fast rename would otherwise lose the accounting data).
    fn tree_size<'a>(
        &'a self,
        path: &'a AbsolutePath,
    ) -> Pin<Box<dyn Future<Output = Result<u64, UseCaseError>> + Send + 'a>> {
        Box::pin(async move {
            let metadata = match tokio_fs::symlink_metadata(path.to_path_buf()).await {
                Ok(m) => m,
                Err(e) => return Err(UseCaseError::FileNotFound(e.to_string())),
            };

            if !metadata.is_dir() {
                return Ok(metadata.len());
            }

            let mut total = 0u64;
            let mut entries = match tokio_fs::read_dir(path.to_path_buf()).await {
                Ok(entries) => entries,
                Err(e) => return Err(UseCaseError::FileSystemError(e.to_string())),
            };
            while let Some(entry) = entries
                .next_entry()
                .await
                .map_err(|e| UseCaseError::FileSystemError(e.to_string()))?
            {
                let file_type = entry
                    .file_type()
                    .await
                    .map_err(|e| UseCaseError::FileSystemError(e.to_string()))?;
                if file_type.is_symlink() {
                    continue;
                }
                let child = AbsolutePath::from(entry.path());
                if file_type.is_dir() {
                    total += self.tree_size(&child).await?;
                } else {
                    let size = entry
                        .metadata()
                        .await
                        .map_err(|e| UseCaseError::FileSystemError(e.to_string()))?
                        .len();
                    total += size;
                }
            }
            Ok(total)
        })
    }

    /// Recursively copies `from` to `to`, preserving the directory
    /// structure including empty subdirectories. Regular file sources are
    /// delegated to [`StdFileSystem::copy_file`]; symlinks and junctions
    /// inside the tree are skipped (never followed).
    fn copy_tree_inner<'a>(
        &'a self,
        from: &'a AbsolutePath,
        to: &'a AbsolutePath,
    ) -> Pin<Box<dyn Future<Output = Result<u64, UseCaseError>> + Send + 'a>> {
        Box::pin(async move {
            let metadata = match tokio_fs::symlink_metadata(from.to_path_buf()).await {
                Ok(m) => m,
                Err(e) => return Err(UseCaseError::FileNotFound(e.to_string())),
            };

            if !metadata.is_dir() {
                return self.copy_file(from, to).await;
            }

            tokio_fs::create_dir_all(to.to_path_buf())
                .await
                .map_err(|e| UseCaseError::FileSystemError(e.to_string()))?;

            let mut total = 0u64;
            let mut entries = match tokio_fs::read_dir(from.to_path_buf()).await {
                Ok(entries) => entries,
                Err(e) => return Err(UseCaseError::FileSystemError(e.to_string())),
            };
            while let Some(entry) = entries
                .next_entry()
                .await
                .map_err(|e| UseCaseError::FileSystemError(e.to_string()))?
            {
                let file_type = entry
                    .file_type()
                    .await
                    .map_err(|e| UseCaseError::FileSystemError(e.to_string()))?;
                if file_type.is_symlink() {
                    continue;
                }
                let name = entry.file_name().to_string_lossy().to_string();
                let child_from = AbsolutePath::from(entry.path());
                let child_to = to.join(name);
                if file_type.is_dir() {
                    total += self.copy_tree_inner(&child_from, &child_to).await?;
                } else {
                    total += self.copy_file(&child_from, &child_to).await?;
                }
            }
            Ok(total)
        })
    }
}

#[async_trait]
impl FileSystem for StdFileSystem {
    async fn exists(&self, path: &AbsolutePath) -> Result<bool, UseCaseError> {
        // use to_path_buf() for reliable conversion
        Ok(tokio_fs::metadata(path.to_path_buf()).await.is_ok())
    }

    /// Returns the size of a file in bytes.
    async fn get_file_size(&self, path: &AbsolutePath) -> Result<u64, UseCaseError> {
        let metadata = tokio_fs::metadata(path.to_path_buf())
            .await
            .map_err(|e| UseCaseError::FileNotFound(e.to_string()))?;
        Ok(metadata.len())
    }

    async fn move_file(&self, from: &AbsolutePath, to: &AbsolutePath) -> Result<u64, UseCaseError> {
        let metadata = tokio_fs::metadata(from.to_path_buf())
            .await
            .map_err(|e| UseCaseError::FileNotFound(e.to_string()))?;
        let size = metadata.len();

        // Try rename first (fast, works on same drive)
        match tokio_fs::rename(from.to_path_buf(), to.to_path_buf()).await {
            Ok(()) => Ok(size),
            Err(_) => {
                // Cross-drive move: copy + delete
                tokio_fs::copy(from.to_path_buf(), to.to_path_buf())
                    .await
                    .map_err(|e| UseCaseError::FileSystemError(e.to_string()))?;
                tokio_fs::remove_file(from.to_path_buf())
                    .await
                    .map_err(|e| UseCaseError::FileSystemError(e.to_string()))?;
                Ok(size)
            }
        }
    }

    async fn copy_file(&self, from: &AbsolutePath, to: &AbsolutePath) -> Result<u64, UseCaseError> {
        let metadata = tokio_fs::metadata(from.to_path_buf())
            .await
            .map_err(|e| UseCaseError::FileNotFound(e.to_string()))?;
        let size = metadata.len();
        // Perform the copy
        tokio_fs::copy(from.to_path_buf(), to.to_path_buf())
            .await
            .map_err(|e| UseCaseError::FileSystemError(e.to_string()))?;
        Ok(size)
    }

    async fn is_dir(&self, path: &AbsolutePath) -> Result<bool, UseCaseError> {
        match tokio_fs::symlink_metadata(path.to_path_buf()).await {
            Ok(metadata) => Ok(metadata.is_dir()),
            // Missing paths are not a directory; other errors (e.g.
            // permission denied) are surfaced to the caller.
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(UseCaseError::FileSystemError(e.to_string())),
        }
    }

    async fn copy_tree(&self, from: &AbsolutePath, to: &AbsolutePath) -> Result<u64, UseCaseError> {
        self.copy_tree_inner(from, to).await
    }

    async fn move_tree(&self, from: &AbsolutePath, to: &AbsolutePath) -> Result<u64, UseCaseError> {
        // Files and symlinks use the simple single-item path.
        if !self.is_dir(from).await? {
            return self.move_file(from, to).await;
        }

        // Measure the size before moving: after a successful rename the
        // source no longer exists and the history record would be lost.
        let size = self.tree_size(from).await?;

        // Fast path: same-volume rename.
        if tokio_fs::rename(from.to_path_buf(), to.to_path_buf())
            .await
            .is_ok()
        {
            return Ok(size);
        }

        // Cross-drive (or otherwise unsupported) rename: copy then delete.
        self.copy_tree(from, to).await?;
        tokio_fs::remove_dir_all(from.to_path_buf())
            .await
            .map_err(|e| UseCaseError::FileSystemError(e.to_string()))?;
        Ok(size)
    }

    async fn delete_file(&self, path: &AbsolutePath) -> Result<(), UseCaseError> {
        tokio_fs::remove_file(path.to_path_buf())
            .await
            .map_err(|e| UseCaseError::FileSystemError(e.to_string()))
    }

    async fn rename_file(
        &self,
        from: &AbsolutePath,
        to: &AbsolutePath,
    ) -> Result<(), UseCaseError> {
        tokio_fs::rename(from.to_path_buf(), to.to_path_buf())
            .await
            .map_err(|e| UseCaseError::FileSystemError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_get_file_size() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");

        {
            let mut file = File::create(&file_path).unwrap();
            writeln!(file, "Hello World").unwrap();
        }

        let fs = StdFileSystem;
        // pass the &str directly
        let path = AbsolutePath::new(file_path.to_str().unwrap()).unwrap();
        let size = fs.get_file_size(&path).await.unwrap();

        assert_eq!(size, 12); // "Hello World\n"
    }

    #[tokio::test]
    async fn test_exists() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        File::create(&file_path).unwrap();

        let fs = StdFileSystem;
        let exists_path = AbsolutePath::new(file_path.to_str().unwrap()).unwrap();
        assert!(fs.exists(&exists_path).await.unwrap());

        let not_exists_path =
            AbsolutePath::new(dir.path().join("nonexistent.txt").to_str().unwrap()).unwrap();
        assert!(!fs.exists(&not_exists_path).await.unwrap());
    }

    #[tokio::test]
    async fn test_is_dir_file_and_folder() {
        let dir = tempdir().unwrap();
        let folder_path = dir.path().join("subdir");
        std::fs::create_dir(&folder_path).unwrap();
        let file_path = dir.path().join("file.txt");
        std::fs::write(&file_path, "data").unwrap();

        let fs = StdFileSystem;

        let folder = AbsolutePath::new(folder_path.to_str().unwrap()).unwrap();
        let file = AbsolutePath::new(file_path.to_str().unwrap()).unwrap();
        let missing = AbsolutePath::new(dir.path().join("nope").to_str().unwrap()).unwrap();

        assert!(fs.is_dir(&folder).await.unwrap());
        assert!(!fs.is_dir(&file).await.unwrap());
        assert!(!fs.is_dir(&missing).await.unwrap());
    }

    #[tokio::test]
    async fn test_copy_tree_nested_directory() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("src");
        std::fs::create_dir_all(src.join("sub")).unwrap();
        std::fs::create_dir(src.join("empty")).unwrap();
        std::fs::write(src.join("a.txt"), "aaa").unwrap();
        std::fs::write(src.join("sub").join("b.txt"), "bbbbbb").unwrap();

        let fs = StdFileSystem;
        let from = AbsolutePath::new(src.to_str().unwrap()).unwrap();
        let to = AbsolutePath::new(dir.path().join("dst").to_str().unwrap()).unwrap();

        let size = fs.copy_tree(&from, &to).await.unwrap();
        assert_eq!(size, 9); // "aaa" (3) + "bbbbbb" (6)

        assert!(to.join("a.txt").to_path_buf().exists());
        assert!(to.join("sub").join("b.txt").to_path_buf().exists());
        assert!(to.join("empty").to_path_buf().is_dir());
        assert!(from.join("a.txt").to_path_buf().exists());
    }

    #[tokio::test]
    async fn test_copy_tree_single_file() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("file.txt");
        std::fs::write(&src, "hello").unwrap();

        let fs = StdFileSystem;
        let from = AbsolutePath::new(src.to_str().unwrap()).unwrap();
        let to = AbsolutePath::new(dir.path().join("copy.txt").to_str().unwrap()).unwrap();

        let size = fs.copy_tree(&from, &to).await.unwrap();
        assert_eq!(size, 5);
        assert!(from.to_path_buf().exists());
        assert!(to.to_path_buf().exists());
    }

    #[tokio::test]
    async fn test_move_tree_directory_same_volume() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("src");
        std::fs::create_dir_all(src.join("sub")).unwrap();
        std::fs::write(src.join("a.txt"), "aaaa").unwrap();
        std::fs::write(src.join("sub").join("b.txt"), "bbbb").unwrap();

        let fs = StdFileSystem;
        let from = AbsolutePath::new(src.to_str().unwrap()).unwrap();
        let to = AbsolutePath::new(dir.path().join("dst").to_str().unwrap()).unwrap();

        let size = fs.move_tree(&from, &to).await.unwrap();
        assert_eq!(size, 8);
        assert!(!from.to_path_buf().exists());
        assert!(to.join("a.txt").to_path_buf().exists());
        assert!(to.join("sub").join("b.txt").to_path_buf().exists());
    }

    #[tokio::test]
    async fn test_move_tree_directory_fallback_copy_delete() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("src");
        std::fs::create_dir(&src).unwrap();
        std::fs::write(src.join("a.txt"), "data").unwrap();

        // An existing non-empty destination directory forces rename to fail,
        // exercising the same copy-then-delete fallback a cross-drive
        // EXDEV error triggers.
        let dst = dir.path().join("dst");
        std::fs::create_dir(&dst).unwrap();
        std::fs::write(dst.join("existing.txt"), "keep").unwrap();

        let fs = StdFileSystem;
        let from = AbsolutePath::new(src.to_str().unwrap()).unwrap();
        let to = AbsolutePath::new(dst.to_str().unwrap()).unwrap();

        let size = fs.move_tree(&from, &to).await.unwrap();
        assert_eq!(size, 4); // "data"
        assert!(!from.to_path_buf().exists());

        assert!(to.join("existing.txt").to_path_buf().exists());
        assert!(to.join("a.txt").to_path_buf().exists());
    }
}
