//! ManageFoldersUseCase - add, remove, rename, reorder folders.
//!
//! # Responsibility
//! Handles CRUD operations for configured folders. Each method modifies the
//! persisted configuration via the `ConfigurationRepository` port.
//!
//! # Design Decisions
//! - `rename_folder` currently loads the entire folder list, modifies the target,
//!   and saves the whole list back. This is acceptable for small numbers of folders.
//!   For larger sets, a dedicated `update` method on the repository should be added.
//! - `toggle_favorite` is a stub awaiting the SQLite migration (see TASK-015).

use async_trait::async_trait;
use std::sync::Arc;
use quicksort_domain::{Folder, FolderId};
use crate::errors::UseCaseError;
use crate::ports::outbound::ConfigurationRepository;
use crate::ports::inbound::ManageFolders;

/// Use case for managing the folder configuration.
pub struct ManageFoldersUseCase {
    config_repo: Arc<dyn ConfigurationRepository>,
}

impl ManageFoldersUseCase {
    /// Creates a new instance backed by the given repository.
    pub fn new(config_repo: Arc<dyn ConfigurationRepository>) -> Self {
        Self { config_repo }
    }
}

#[async_trait]
impl ManageFolders for ManageFoldersUseCase {
    /// Adds a new folder to the configuration.
    ///
    /// # Errors
    /// Returns `RepositoryError` if the underlying storage fails.
    async fn add_folder(&self, folder: Folder) -> Result<(), UseCaseError> {
        self.config_repo
            .add(folder)
            .await
            // OLD: no error mapping – relied on `?` with implicit conversion
            // NEW: explicit mapping from anyhow::Error to UseCaseError
            .map_err(|e| UseCaseError::RepositoryError(e.to_string()))
    }

    /// Removes a folder by its unique identifier.
    ///
    /// # Errors
    /// Returns `RepositoryError` on storage failure.
    /// Does not return an error if the folder does not exist (idempotent).
    async fn remove_folder(&self, id: FolderId) -> Result<(), UseCaseError> {
        self.config_repo
            .remove(&id)
            .await
            .map_err(|e| UseCaseError::RepositoryError(e.to_string()))
    }

    /// Renames an existing folder.
    ///
    /// # Implementation Note
    /// Currently loads the entire folder list, finds the target, updates its name,
    /// and persists the whole list. For a large number of folders, consider adding
    /// an `update` method to `ConfigurationRepository` that modifies a single entry
    /// atomically.
    async fn rename_folder(&self, id: FolderId, new_name: String) -> Result<(), UseCaseError> {
        // Load current state
        let mut folders = self.config_repo
            .load_all()
            .await
            .map_err(|e| UseCaseError::RepositoryError(e.to_string()))?;

        // Find and update the target folder
        let folder = folders
            .iter_mut()
            .find(|f| f.id == id)
            .ok_or_else(|| UseCaseError::FolderNotFound(id.to_string()))?;

        // OLD: direct field mutation
        // NEW: preserve the original comment for clarity
        folder.name = new_name;

        // Persist the updated list
        self.config_repo
            .save_all(&folders)
            .await
            .map_err(|e| UseCaseError::RepositoryError(e.to_string()))
    }

    /// Toggles the favorite status of a folder.
    ///
    /// **Current status:** Stub implementation.
    ///
    /// This functionality requires additional fields (`is_favorite`, `sort_order`)
    /// in the `Folder` entity, which will be introduced during the SQLite migration
    /// (see TASK-015). Once those fields exist, this method will delegate to a
    /// `ConfigurationRepository::update` method that modifies a single folder.
    // OLD: Russian TODO comment
    // NEW: Translated to English with a reference to the tracking task
    // TODO: TASK-015 – implement toggle_favorite after Folder gains is_favorite field
    async fn toggle_favorite(&self, _id: FolderId) -> Result<(), UseCaseError> {
        Ok(())
    }
}