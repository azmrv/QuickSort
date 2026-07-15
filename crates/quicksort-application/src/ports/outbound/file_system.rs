//! Outbound port for file system operations.
//!
//! This port defines the interface that infrastructure must implement
//! to perform actual file operations (move, copy, delete, rename).
//! It is used by `ExecuteOperationUseCase` and `UndoOperationUseCase`.
//!
//! # Design Decision
//! The port uses `quicksort_domain::WindowsPath` instead of `std::path::Path`
//! to ensure that only validated domain paths are accepted. This prevents
//! malformed or unsafe paths from reaching the file system adapter.

use async_trait::async_trait;
use quicksort_domain::WindowsPath;
use crate::errors::UseCaseError;

/// Port for performing file operations on the local file system.
///
/// All methods are asynchronous and return `UseCaseError` to isolate
/// the Application layer from infrastructure-specific error types.
#[async_trait]
pub trait FileSystem: Send + Sync {
    /// Checks whether a file or directory exists at the given path.
    ///
    /// # Returns
    /// `true` if the path exists, `false` otherwise.
    ///
    /// # Errors
    /// Returns `FileSystemError` if the check fails (e.g., permission denied
    /// while traversing a parent directory).
    async fn exists(&self, path: &WindowsPath) -> Result<bool, UseCaseError>;

    /// Returns the size of a file in bytes.
    ///
    /// # Errors
    /// Returns `FileNotFound` if the path does not exist or is a directory.
    /// Returns `FileSystemError` if metadata retrieval fails.
    // OLD: Получить размер файла в байтах.
    // NEW: English comment as per project standards
    async fn get_file_size(&self, path: &WindowsPath) -> Result<u64, UseCaseError>;

    /// Moves a file from `from` to `to`.
    ///
    /// # Returns
    /// The size of the moved file in bytes.
    ///
    /// # Implementation Notes
    /// For cross-volume moves (EXDEV error), the implementation should
    /// fall back to copy + delete. This is handled by the infrastructure
    /// adapter (`StdFileSystem`).
    ///
    /// # Errors
    /// Returns `FileNotFound` if the source does not exist.
    /// Returns `PermissionDenied` if access is restricted.
    /// Returns `Conflict` if the destination exists and the operation
    /// is not configured to overwrite.
    async fn move_file(&self, from: &WindowsPath, to: &WindowsPath) -> Result<u64, UseCaseError>;

    /// Copies a file from `from` to `to`, returning the file size in bytes.
    ///
    /// # Returns
    /// The size of the copied file in bytes.
    ///
    /// # Errors
    /// Returns `FileNotFound` if the source does not exist.
    /// Returns `FileSystemError` on I/O failure (disk full, etc.).
    async fn copy_file(&self, from: &WindowsPath, to: &WindowsPath) -> Result<u64, UseCaseError>;

    /// Deletes a file at the given path.
    ///
    /// The method does **not** return a file size because `std::fs::remove_file`
    /// does not provide metadata, and obtaining it beforehand would require
    /// an additional `get_file_size` call. The `Operation` aggregate records
    /// the size through the `complete` method's `bytes_processed` parameter,
    /// which is supplied by the Use Case after calling `get_file_size`.
    ///
    /// # Errors
    /// Returns `FileNotFound` if the path does not exist.
    /// Returns `PermissionDenied` if the file cannot be removed.
    async fn delete_file(&self, path: &WindowsPath) -> Result<(), UseCaseError>;

    /// Renames a file or directory from `from` to `to`.
    ///
    /// For regular files, this is equivalent to a move within the same
    /// volume. The method does **not** return a file size for the same
    /// reason as `delete_file`: `std::fs::rename` does not provide it,
    /// and the Use Case is responsible for tracking sizes separately.
    ///
    /// # Errors
    /// Returns `FileNotFound` if the source does not exist.
    /// Returns `PermissionDenied` if access is restricted.
    async fn rename_file(&self, from: &WindowsPath, to: &WindowsPath) -> Result<(), UseCaseError>;
}