//! Outbound port for file system operations.

use async_trait::async_trait;
use quicksort_domain::WindowsPath;
use crate::errors::UseCaseError;

#[async_trait]
pub trait FileSystem: Send + Sync {
    async fn exists(&self, path: &WindowsPath) -> Result<bool, UseCaseError>;
    /// Получить размер файла в байтах.
    async fn get_file_size(&self, path: &WindowsPath) -> Result<u64, UseCaseError>;
    async fn move_file(&self, from: &WindowsPath, to: &WindowsPath) -> Result<u64, UseCaseError>;
    async fn copy_file(&self, from: &WindowsPath, to: &WindowsPath) -> Result<u64, UseCaseError>;
    async fn delete_file(&self, path: &WindowsPath) -> Result<(), UseCaseError>;
    async fn rename_file(&self, from: &WindowsPath, to: &WindowsPath) -> Result<(), UseCaseError>;
}