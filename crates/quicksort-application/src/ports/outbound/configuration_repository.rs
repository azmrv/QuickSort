//! Outbound port for configuration storage (folders, settings).
//!
//! This port defines the interface that infrastructure must implement
//! to persist and retrieve folder configurations. It is used by
//! `GetFoldersUseCase` and `ManageFoldersUseCase`.
//!
//! # Design Decision
//! The repository operates on domain entities (`Folder`, `FolderId`)
//! rather than DTOs or raw data. This keeps the Application layer
//! independent of serialization formats (JSON, SQLite, etc.).

use async_trait::async_trait;
use quicksort_domain::{Folder, FolderId};
use crate::errors::UseCaseError;

/// Persistence port for folder configuration.
///
/// All methods return `UseCaseError` to insulate the Application
/// layer from infrastructure-specific error types (e.g., `io::Error`,
/// `serde_json::Error`).
#[async_trait]
pub trait ConfigurationRepository: Send + Sync {
    /// Loads all stored folders from the configuration.
    ///
    /// # Returns
    /// A vector of all folders, or an empty vector if none exist.
    ///
    /// # Errors
    /// Returns `RepositoryError` if the underlying storage cannot be read
    /// (e.g., file not found, malformed data, I/O failure).
    async fn load_all(&self) -> Result<Vec<Folder>, UseCaseError>;

    /// Saves the complete list of folders, replacing any existing data.
    ///
    /// This is an atomic operation: either all folders are saved, or
    /// none are (the repository implementation must guarantee this).
    ///
    /// # Errors
    /// Returns `RepositoryError` if the write operation fails.
    async fn save_all(&self, folders: &[Folder]) -> Result<(), UseCaseError>;

    /// Adds a single folder to the configuration.
    ///
    /// The repository is responsible for checking uniqueness of the
    /// folder ID and preventing duplicates.
    ///
    /// # Errors
    /// Returns `RepositoryError` if the save operation fails.
    async fn add(&self, folder: Folder) -> Result<(), UseCaseError>;

    /// Removes a folder by its unique identifier.
    ///
    /// If the folder does not exist, the operation is idempotent and
    /// does not return an error.
    ///
    /// # Errors
    /// Returns `RepositoryError` if the underlying storage fails.
    async fn remove(&self, id: &FolderId) -> Result<(), UseCaseError>;

    /// Finds a folder by its unique identifier.
    ///
    /// # Returns
    /// `Some(Folder)` if found, `None` otherwise.
    ///
    /// # Errors
    /// Returns `RepositoryError` on storage failure.
    async fn find_by_id(&self, id: &FolderId) -> Result<Option<Folder>, UseCaseError>;

    /// Finds a folder by its full filesystem path.
    ///
    /// # Returns
    /// `Some(Folder)` if a folder with the given path exists, `None` otherwise.
    ///
    /// # Errors
    /// Returns `RepositoryError` on storage failure.
    async fn find_by_path(&self, path: &str) -> Result<Option<Folder>, UseCaseError>;

    /// Returns the ID of the default folder (e.g., "Documents").
    ///
    /// This is used to provide a fallback when no target folder is
    /// explicitly specified by the user.
    ///
    /// # Errors
    /// Returns `FolderNotFound` if no default folder is configured.
    async fn get_default_folder_id(&self) -> Result<FolderId, UseCaseError>;
}