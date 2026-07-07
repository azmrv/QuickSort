use std::path::Path;
use async_trait::async_trait;
use tokio::fs as tokio_fs;

use quicksort_domain::WindowsPath;
use quicksort_application::ports::outbound::FileSystem;
use quicksort_application::errors::UseCaseError;

pub struct StdFileSystem;

impl StdFileSystem {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl FileSystem for StdFileSystem {
    async fn exists(&self, path: &WindowsPath) -> Result<bool, UseCaseError> {
        Ok(tokio_fs::metadata(Path::new(path.as_str())).await.is_ok())
    }

    async fn move_file(&self, from: &WindowsPath, to: &WindowsPath) -> Result<u64, UseCaseError> {
        let metadata = tokio_fs::metadata(Path::new(from.as_str()))
            .await
            .map_err(|e| UseCaseError::FileNotFound(e.to_string()))?;
        let size = metadata.len();
        tokio_fs::rename(Path::new(from.as_str()), Path::new(to.as_str()))
            .await
            .map_err(|e| UseCaseError::FileSystemError(e.to_string()))?;
        Ok(size)
    }

    async fn copy_file(&self, from: &WindowsPath, to: &WindowsPath) -> Result<u64, UseCaseError> {
        let metadata = tokio_fs::metadata(Path::new(from.as_str()))
            .await
            .map_err(|e| UseCaseError::FileNotFound(e.to_string()))?;
        let size = metadata.len();
        tokio_fs::copy(Path::new(from.as_str()), Path::new(to.as_str()))
            .await
            .map_err(|e| UseCaseError::FileSystemError(e.to_string()))?;
        Ok(size)
    }

    async fn delete_file(&self, path: &WindowsPath) -> Result<(), UseCaseError> {
        tokio_fs::remove_file(Path::new(path.as_str()))
            .await
            .map_err(|e| UseCaseError::FileSystemError(e.to_string()))
    }

    async fn rename_file(&self, from: &WindowsPath, to: &WindowsPath) -> Result<(), UseCaseError> {
        tokio_fs::rename(Path::new(from.as_str()), Path::new(to.as_str()))
            .await
            .map_err(|e| UseCaseError::FileSystemError(e.to_string()))
    }
}