//! Standard implementation of the FileSystem port using tokio::fs.

use std::path::PathBuf;
use async_trait::async_trait;
use tokio::fs as tokio_fs;

use quicksort_domain::WindowsPath;
use quicksort_application::ports::outbound::FileSystem;
use quicksort_application::errors::UseCaseError;

/// Real file system implementation backed by tokio.
pub struct StdFileSystem;

impl StdFileSystem {
    /// Creates a new StdFileSystem.
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl FileSystem for StdFileSystem {
    async fn exists(&self, path: &WindowsPath) -> Result<bool, UseCaseError> {
        // OLD: tokio_fs::metadata(Path::new(path.as_str()))
        // NEW: use to_path_buf() for reliable conversion
        Ok(tokio_fs::metadata(path.to_path_buf()).await.is_ok())
    }

    /// Returns the size of a file in bytes.
    async fn get_file_size(&self, path: &WindowsPath) -> Result<u64, UseCaseError> {
        let metadata = tokio_fs::metadata(path.to_path_buf())
            .await
            .map_err(|e| UseCaseError::FileNotFound(e.to_string()))?;
        Ok(metadata.len())
    }

    async fn move_file(&self, from: &WindowsPath, to: &WindowsPath) -> Result<u64, UseCaseError> {
        let metadata = tokio_fs::metadata(from.to_path_buf())
            .await
            .map_err(|e| UseCaseError::FileNotFound(e.to_string()))?;
        let size = metadata.len();
        // Perform the move
        tokio_fs::rename(from.to_path_buf(), to.to_path_buf())
            .await
            .map_err(|e| UseCaseError::FileSystemError(e.to_string()))?;
        // If the move was successful, return the original file size
        Ok(size)
    }

    async fn copy_file(&self, from: &WindowsPath, to: &WindowsPath) -> Result<u64, UseCaseError> {
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

    async fn delete_file(&self, path: &WindowsPath) -> Result<(), UseCaseError> {
        tokio_fs::remove_file(path.to_path_buf())
            .await
            .map_err(|e| UseCaseError::FileSystemError(e.to_string()))
    }

    async fn rename_file(&self, from: &WindowsPath, to: &WindowsPath) -> Result<(), UseCaseError> {
        tokio_fs::rename(from.to_path_buf(), to.to_path_buf())
            .await
            .map_err(|e| UseCaseError::FileSystemError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
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
        // OLD: WindowsPath::new(file_path.to_str().unwrap().to_string())
        // NEW: pass the &str directly
        let path = WindowsPath::new(file_path.to_str().unwrap()).unwrap();
        let size = fs.get_file_size(&path).await.unwrap();
        
        assert_eq!(size, 11); // "Hello World\n"
    }

    #[tokio::test]
    async fn test_exists() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        File::create(&file_path).unwrap();
        
        let fs = StdFileSystem;
        let exists_path = WindowsPath::new(file_path.to_str().unwrap()).unwrap();
        assert!(fs.exists(&exists_path).await.unwrap());
        
        let not_exists_path = WindowsPath::new(
            dir.path().join("nonexistent.txt").to_str().unwrap()
        ).unwrap();
        assert!(!fs.exists(&not_exists_path).await.unwrap());
    }
}