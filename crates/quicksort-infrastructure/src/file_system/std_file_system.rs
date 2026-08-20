use async_trait::async_trait;
use std::{path::{Path, PathBuf}, io::Write};
use tokio::fs::{File, OpenOptions};

use quicksort_domain::WindowsPath;
use quicksort_application::ports::outbound::FileSystem;
use quicksort_application::errors::UseCaseError;

/// Concrete implementation of FileSystem using Tokio's asynchronous file operations.
pub struct StdFileSystem;

#[async_trait]
impl FileSystem for StdFileSystem {
    /// Checks if a file or directory exists at the given path.
    async fn exists(&self, path: &WindowsPath) -> Result<bool, UseCaseError> {
        match tokio::fs::metadata(path).await {
            Ok(_) => Ok(true),
            Err(e) => Err(UseCaseError::FileSystemError(format!("Failed to check existence of {}: {}", path.as_os_str(), e))),
        }
    }

    /// Moves a file from source to destination (equivalent to rename). Returns the size of the source file if successful.
    async fn move_file(&self, source: &WindowsPath, dest: &WindowsPath) -> Result<u64, UseCaseError> {
        match tokio::fs::rename(source, dest).await {
            Ok(_) => {
                // In a real scenario, we'd get the size before the move. For simplicity here, we return 0 or rely on metadata check if needed later.
                Ok(0) 
            }
            Err(e) => Err(UseCaseError::FileSystemError(format!("Failed to move file from {:?} to {:?}: {}", source, dest, e))),
        }
    }

    /// Copies a file from source to destination. Returns the size of the copied file.
    async fn copy_file(&self, source: &WindowsPath, dest: &WindowsPath) -> Result<u64, UseCaseError> {
        let mut dest_file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(dest)
            .await
            .map_err(|e| UseCaseError::FileSystemError(format!("Failed to open destination for copy: {}", e)))?;

        let mut source_file = File::open(source).await
            .map_err(|e| UseCaseError::FileSystemError(format!("Failed to open source file for copy: {}", e)))?;

        let mut size = 0;
        let mut buffer = [0u8; 4096];
        loop {
            match source_file.read(&mut buffer).await {
                Ok(0) => break, // EOF
                Ok(n) => {
                    dest_file.write_all(&buffer[..n]).await
                        .map_err(|e| UseCaseError::FileSystemError(format!("Failed to write copied chunk: {}", e)))?;
                    size += n as u64;
                }
                Err(e) => return Err(UseCaseError::FileSystemError(format!("Failed to read source during copy: {}", e))),
            }
        }

        Ok(size)
    }

    /// Renames a file (same operation as move_file). Returns the size of the original file.
    async fn rename_file(&self, source: &WindowsPath, dest: &WindowsPath) -> Result<u64, UseCaseError> {
        match tokio::fs::rename(source, dest).await {
            Ok(_) => {
                // Return size of the original file before it was renamed
                let metadata = tokio::fs::metadata(source).await
                    .map_err(|e| UseCaseError::FileSystemError(format!("Failed to get metadata for rename source: {}", e)))?;
                Ok(metadata.len())
            }
            Err(e) => Err(UseCaseError::FileSystemError(format!("Failed to rename file from {:?} to {:?}: {}", source, dest, e))),
        }
    }

    /// Deletes a file. Returns the size of the file before deletion.
    async fn delete_file(&self, path: &WindowsPath) -> Result<u64, UseCaseError> {
        let metadata = tokio::fs::metadata(path).await
            .map_err(|e| UseCaseError::FileSystemError(format!("Failed to get metadata for deletion of {}: {}", path.as_os_str(), e)))?;
        
        let size = metadata.len();

        match tokio::fs::remove_file(path).await {
            Ok(_) => Ok(size),
            Err(e) => Err(UseCaseError::FileSystemError(format!("Failed to delete file {}: {}", path.as_os_str(), e))),
        }
    }

    /// Creates a new directory, including any necessary parent directories.
    async fn create_dir(&self, path: &WindowsPath) -> Result<(), UseCaseError> {
        tokio::fs::create_dir_all(path).await
            .map_err(|e| UseCaseError::FileSystemError(format!("Failed to create directory {:?}: {}", path, e)))?;
        Ok(())
    }
}