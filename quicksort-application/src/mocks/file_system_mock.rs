// mocks/file_system_mock.rs
use async_trait::async_trait;
use crate::application::ports::FileSystemPort;
use crate::domain::{WindowsPath, DomainError};
use std::sync::Mutex;

#[derive(Debug)]
pub struct FileSystemMock {
    // Simulate file system state if needed for complex mocks
}

impl FileSystemMock {
    pub fn new() -> Self {
        FileSystemMock {}
    }
}

#[async_trait]
impl FileSystemPort for FileSystemMock {
    async fn move_files(&self, src: &WindowsPath, target: &WindowsPath) -> Result<OperationExecutionResult, DomainError> {
        // Mock successful move operation
        println!("MOCK: Moving file from {} to {}", src.as_str(), target.as_str());
        Ok(OperationExecutionResult { files: 1, bytes: 1024 })
    }

    async fn copy_files(&self, src: &WindowsPath, target: &WindowsPath) -> Result<OperationExecutionResult, DomainError> {
        // Mock successful copy operation
        println!("MOCK: Copying file from {} to {}", src.as_str(), target.as_str());
        Ok(OperationExecutionResult { files: 1, bytes: 2048 })
    }

    async fn delete_path(&self, path: &WindowsPath) -> Result<OperationExecutionResult, DomainError> {
        // Mock successful delete operation
        println!("MOCK: Deleting path {}", path.as_str());
        Ok(OperationExecutionResult { files: 1, bytes: 0 })
    }

    async fn rename_path(&self, src: &WindowsPath, target: &WindowsPath) -> Result<OperationExecutionResult, DomainError> {
        // Mock successful rename operation
        println!("MOCK: Renaming {} to {}", src.as_str(), target.as_str());
        Ok(OperationExecutionResult { files: 1, bytes: 0 })
    }
}