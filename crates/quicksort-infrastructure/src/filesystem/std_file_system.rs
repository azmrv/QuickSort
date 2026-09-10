//! Standard implementation of the FileSystem port using tokio::fs.

use std::future::Future;
use std::pin::Pin;

use async_trait::async_trait;
use tokio::fs as tokio_fs;

use quicksort_application::dtos::FolderMetadata;
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
    /// Returns the Windows extended-length (`\\?\`) form of the path so
    /// that paths longer than the legacy 260-char `MAX_PATH` limit work.
    ///
    /// On non-Windows platforms this is the identity conversion. Paths
    /// that already carry the verbatim prefix, UNC paths
    /// (`\\server\share\...` → `\\?\UNC\server\share\...`) and
    /// drive-absolute paths (`C:\...` → `\\?\C:\...`) are handled;
    /// root-relative paths (`\foo`) are returned unchanged because a
    /// verbatim prefix is invalid for them.
    #[cfg(target_os = "windows")]
    fn extended_path(path: &AbsolutePath) -> std::path::PathBuf {
        let raw = path.to_path_buf();
        let s = raw.to_string_lossy();
        // Already verbatim-prefixed (or a device path) — use as-is.
        if s.starts_with(r"\\?\") {
            return raw;
        }
        // UNC path: \\server\share\... → \\?\UNC\server\share\...
        if s.starts_with(r"\\") {
            return std::path::PathBuf::from(format!(r"\\?\UNC\{}", &s[2..]));
        }
        // Drive-absolute path (X:\...): prefix it.
        if s.len() >= 3 && s.as_bytes()[1] == b':' {
            return std::path::PathBuf::from(format!(r"\\?\{s}"));
        }
        // Root-relative path — the verbatim prefix cannot be applied.
        raw
    }

    #[cfg(not(target_os = "windows"))]
    fn extended_path(path: &AbsolutePath) -> std::path::PathBuf {
        path.to_path_buf()
    }

    /// Recursively computes `(total_size, item_count)` of all regular files
    /// under `path`. Symlinks and junctions are not followed.
    fn tree_stats<'a>(
        &'a self,
        path: &'a AbsolutePath,
    ) -> Pin<Box<dyn Future<Output = Result<(u64, u64), UseCaseError>> + Send + 'a>> {
        Box::pin(async move {
            let metadata = match tokio_fs::symlink_metadata(Self::extended_path(path)).await {
                Ok(m) => m,
                Err(e) => return Err(UseCaseError::FileNotFound(e.to_string())),
            };

            if !metadata.is_dir() {
                return Ok((metadata.len(), 1));
            }

            let mut total = 0u64;
            let mut count = 0u64;
            let mut entries = match tokio_fs::read_dir(Self::extended_path(path)).await {
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
                    let (size, child_count) = self.tree_stats(&child).await?;
                    total += size;
                    count += child_count;
                } else {
                    let size = entry
                        .metadata()
                        .await
                        .map_err(|e| UseCaseError::FileSystemError(e.to_string()))?
                        .len();
                    total += size;
                    count += 1;
                }
            }
            Ok((total, count))
        })
    }

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
            let (total, _) = self.tree_stats(path).await?;
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
            let metadata = match tokio_fs::symlink_metadata(Self::extended_path(from)).await {
                Ok(m) => m,
                Err(e) => return Err(UseCaseError::FileNotFound(e.to_string())),
            };

            if !metadata.is_dir() {
                return self.copy_file(from, to).await;
            }

            tokio_fs::create_dir_all(Self::extended_path(to))
                .await
                .map_err(|e| UseCaseError::FileSystemError(e.to_string()))?;

            let mut total = 0u64;
            let mut entries = match tokio_fs::read_dir(Self::extended_path(from)).await {
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
        Ok(tokio_fs::metadata(Self::extended_path(path)).await.is_ok())
    }

    /// Returns the size of a file in bytes.
    async fn get_file_size(&self, path: &AbsolutePath) -> Result<u64, UseCaseError> {
        let metadata = tokio_fs::metadata(Self::extended_path(path))
            .await
            .map_err(|e| UseCaseError::FileNotFound(e.to_string()))?;
        Ok(metadata.len())
    }

    async fn move_file(&self, from: &AbsolutePath, to: &AbsolutePath) -> Result<u64, UseCaseError> {
        let metadata = tokio_fs::metadata(Self::extended_path(from))
            .await
            .map_err(|e| UseCaseError::FileNotFound(e.to_string()))?;
        let size = metadata.len();

        // Try rename first (fast, works on same drive)
        match tokio_fs::rename(Self::extended_path(from), Self::extended_path(to)).await {
            Ok(()) => Ok(size),
            Err(_) => {
                // Cross-drive move: copy + delete
                tokio_fs::copy(Self::extended_path(from), Self::extended_path(to))
                    .await
                    .map_err(|e| UseCaseError::FileSystemError(e.to_string()))?;
                tokio_fs::remove_file(Self::extended_path(from))
                    .await
                    .map_err(|e| UseCaseError::FileSystemError(e.to_string()))?;
                Ok(size)
            }
        }
    }

    async fn copy_file(&self, from: &AbsolutePath, to: &AbsolutePath) -> Result<u64, UseCaseError> {
        let metadata = tokio_fs::metadata(Self::extended_path(from))
            .await
            .map_err(|e| UseCaseError::FileNotFound(e.to_string()))?;
        let size = metadata.len();
        // Perform the copy
        tokio_fs::copy(Self::extended_path(from), Self::extended_path(to))
            .await
            .map_err(|e| UseCaseError::FileSystemError(e.to_string()))?;
        Ok(size)
    }

    async fn is_dir(&self, path: &AbsolutePath) -> Result<bool, UseCaseError> {
        match tokio_fs::symlink_metadata(Self::extended_path(path)).await {
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
        if tokio_fs::rename(Self::extended_path(from), Self::extended_path(to))
            .await
            .is_ok()
        {
            return Ok(size);
        }

        // Cross-drive (or otherwise unsupported) rename: copy then delete.
        self.copy_tree(from, to).await?;
        tokio_fs::remove_dir_all(Self::extended_path(from))
            .await
            .map_err(|e| UseCaseError::FileSystemError(e.to_string()))?;
        Ok(size)
    }

    async fn delete_file(&self, path: &AbsolutePath) -> Result<(), UseCaseError> {
        tokio_fs::remove_file(Self::extended_path(path))
            .await
            .map_err(|e| UseCaseError::FileSystemError(e.to_string()))
    }

    async fn rename_file(
        &self,
        from: &AbsolutePath,
        to: &AbsolutePath,
    ) -> Result<(), UseCaseError> {
        tokio_fs::rename(Self::extended_path(from), Self::extended_path(to))
            .await
            .map_err(|e| UseCaseError::FileSystemError(e.to_string()))
    }

    async fn generate_unique_path(
        &self,
        path: &AbsolutePath,
    ) -> Result<AbsolutePath, UseCaseError> {
        if !self.exists(path).await? {
            return Ok(path.clone());
        }

        let base = path.to_path_buf();
        let parent = base
            .parent()
            .map(std::path::Path::to_path_buf)
            .unwrap_or_default();
        let file_name = base
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let stem = base
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| file_name.clone());
        let ext = base.extension().map(|e| e.to_string_lossy().into_owned());

        const MAX_ATTEMPTS: u32 = 1000;
        for n in 1..=MAX_ATTEMPTS {
            let candidate_name = match &ext {
                Some(ext) => format!("{stem} ({n}).{ext}"),
                None => format!("{stem} ({n})"),
            };
            let candidate = AbsolutePath::from(parent.join(candidate_name));
            if !self.exists(&candidate).await? {
                return Ok(candidate);
            }
        }

        Err(UseCaseError::Conflict(format!(
            "could not generate a unique path for {} after {} attempts",
            path, MAX_ATTEMPTS
        )))
    }

    async fn folder_metadata(&self, path: &AbsolutePath) -> Result<FolderMetadata, UseCaseError> {
        let metadata = match tokio_fs::symlink_metadata(Self::extended_path(path)).await {
            Ok(m) => m,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Ok(FolderMetadata {
                    exists: false,
                    is_dir: false,
                    total_size: 0,
                    item_count: 0,
                });
            }
            Err(e) => return Err(UseCaseError::FileSystemError(e.to_string())),
        };

        if !metadata.is_dir() {
            return Ok(FolderMetadata {
                exists: true,
                is_dir: false,
                total_size: metadata.len(),
                item_count: 1,
            });
        }

        let (total_size, item_count) = self.tree_stats(path).await?;
        Ok(FolderMetadata {
            exists: true,
            is_dir: true,
            total_size,
            item_count,
        })
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

    #[cfg(target_os = "windows")]
    #[test]
    fn test_extended_path_prefixes() {
        // Drive-absolute path gets the verbatim prefix.
        let drive = AbsolutePath::new(r"C:\Users\me\file.txt").unwrap();
        assert_eq!(
            StdFileSystem::extended_path(&drive).to_string_lossy(),
            r"\\?\C:\Users\me\file.txt"
        );

        // UNC path becomes the extended UNC form.
        let unc = AbsolutePath::new(r"\\server\share\dir").unwrap();
        assert_eq!(
            StdFileSystem::extended_path(&unc).to_string_lossy(),
            r"\\?\UNC\server\share\dir"
        );

        // Already verbatim-prefixed paths are returned unchanged.
        let verbatim = AbsolutePath::new(r"\\?\C:\Users\me\file.txt").unwrap();
        assert_eq!(
            StdFileSystem::extended_path(&verbatim).to_string_lossy(),
            r"\\?\C:\Users\me\file.txt"
        );
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn test_extended_path_identity_on_unix() {
        let path = AbsolutePath::new("/home/me/file.txt").unwrap();
        assert_eq!(
            StdFileSystem::extended_path(&path).to_string_lossy(),
            "/home/me/file.txt"
        );
    }

    #[cfg(target_os = "windows")]
    #[tokio::test]
    async fn test_long_path_exceeds_max_path() {
        let dir = tempdir().unwrap();
        let fs = StdFileSystem;

        // Build a nested path well beyond the legacy 260-char MAX_PATH limit.
        let segment = "a_very_long_directory_segment_that_pads_the_path_";
        let mut deep = dir.path().to_path_buf();
        for i in 0..12 {
            deep = deep.join(format!("{segment}{i:02}"));
        }
        let file = deep.join("payload.txt");
        assert!(file.to_string_lossy().len() > 260);

        // Create the tree through the extended-length prefix.
        std::fs::create_dir_all(StdFileSystem::extended_path(&AbsolutePath::from(
            deep.clone(),
        )))
        .unwrap();
        std::fs::write(
            StdFileSystem::extended_path(&AbsolutePath::from(file.clone())),
            b"long",
        )
        .unwrap();

        let abs = AbsolutePath::from(file);
        assert!(fs.exists(&abs).await.unwrap());
        assert_eq!(fs.get_file_size(&abs).await.unwrap(), 4);
        assert!(!fs.is_dir(&abs).await.unwrap());
    }

    #[tokio::test]
    async fn test_generate_unique_path_free_and_colliding() {
        let dir = tempdir().unwrap();
        let fs = StdFileSystem;

        let target = dir.path().join("report.txt");
        std::fs::write(&target, "v1").unwrap();
        std::fs::write(dir.path().join("report (1).txt"), "v2").unwrap();

        let existing = AbsolutePath::new(target.to_str().unwrap()).unwrap();
        let unique = fs.generate_unique_path(&existing).await.unwrap();
        assert_eq!(unique.to_path_buf(), dir.path().join("report (2).txt"));
    }

    #[tokio::test]
    async fn test_generate_unique_path_dotfile() {
        let dir = tempdir().unwrap();
        let fs = StdFileSystem;

        // Dotfiles have no file stem; the whole name is used as the stem.
        let dotfile = dir.path().join(".gitignore");
        std::fs::write(&dotfile, "target/").unwrap();

        let existing = AbsolutePath::new(dotfile.to_str().unwrap()).unwrap();
        let unique = fs.generate_unique_path(&existing).await.unwrap();
        assert_eq!(unique.to_path_buf(), dir.path().join(".gitignore (1)"));
    }

    #[tokio::test]
    async fn test_folder_metadata() {
        let dir = tempdir().unwrap();
        let fs = StdFileSystem;

        // Missing path: exists=false, zeroed counters, no error.
        let missing = AbsolutePath::new(dir.path().join("nope").to_str().unwrap()).unwrap();
        let meta = fs.folder_metadata(&missing).await.unwrap();
        assert!(!meta.exists);
        assert!(!meta.is_dir);
        assert_eq!(meta.total_size, 0);
        assert_eq!(meta.item_count, 0);

        // Directory tree: sizes and items are summed recursively.
        let tree = dir.path().join("tree");
        std::fs::create_dir_all(tree.join("sub")).unwrap();
        std::fs::write(tree.join("a.txt"), "aaa").unwrap();
        std::fs::write(tree.join("sub").join("b.txt"), "bbbbbb").unwrap();

        let tree_path = AbsolutePath::new(tree.to_str().unwrap()).unwrap();
        let meta = fs.folder_metadata(&tree_path).await.unwrap();
        assert!(meta.exists);
        assert!(meta.is_dir);
        assert_eq!(meta.total_size, 9); // "aaa" (3) + "bbbbbb" (6)
        assert_eq!(meta.item_count, 2);

        // Single file: is_dir=false, one item.
        let file_path = AbsolutePath::new(tree.join("a.txt").to_str().unwrap()).unwrap();
        let meta = fs.folder_metadata(&file_path).await.unwrap();
        assert!(meta.exists);
        assert!(!meta.is_dir);
        assert_eq!(meta.total_size, 3);
        assert_eq!(meta.item_count, 1);
    }
}
