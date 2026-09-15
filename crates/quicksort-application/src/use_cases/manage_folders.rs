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

use crate::errors::UseCaseError;
use crate::ports::inbound::ManageFolders;
use crate::ports::outbound::ConfigurationRepository;
use async_trait::async_trait;
use quicksort_domain::errors::DomainError;
use quicksort_domain::{Folder, FolderId};
use std::sync::Arc;

/// Maximum allowed folder nesting depth (spec `4f_tree_parent_id.md`).
const MAX_FOLDER_DEPTH: usize = 10;

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
    /// Returns `FolderNotFound` when the referenced parent does not exist.
    /// Returns `FolderDepthExceeded` when nesting would exceed the maximum.
    async fn add_folder(&self, folder: Folder) -> Result<(), UseCaseError> {
        if let Some(ref parent_id) = folder.parent_id {
            self.validate_parent_exists_and_depth(parent_id).await?;
        }
        self.config_repo
            .add(folder)
            .await
            // explicit mapping from anyhow::Error to UseCaseError
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
        let mut folders = self
            .config_repo
            .load_all()
            .await
            .map_err(|e| UseCaseError::RepositoryError(e.to_string()))?;

        // Find and update the target folder
        let folder = folders
            .iter_mut()
            .find(|f| f.id == id)
            .ok_or_else(|| UseCaseError::FolderNotFound(id.to_string()))?;

        // preserve the original comment for clarity
        folder.name = new_name;

        // Persist the updated list
        self.config_repo
            .save_all(&folders)
            .await
            .map_err(|e| UseCaseError::RepositoryError(e.to_string()))
    }

    /// Toggles the favorite status of a folder.
    ///
    /// Loads all folders, finds the target by ID, toggles its `favorite` flag,
    /// and persists the updated list.
    async fn toggle_favorite(&self, id: FolderId) -> Result<(), UseCaseError> {
        let mut folders = self
            .config_repo
            .load_all()
            .await
            .map_err(|e| UseCaseError::RepositoryError(e.to_string()))?;

        let folder = folders
            .iter_mut()
            .find(|f| f.id == id)
            .ok_or_else(|| UseCaseError::FolderNotFound(id.to_string()))?;

        folder.toggle_favorite();

        self.config_repo
            .save_all(&folders)
            .await
            .map_err(|e| UseCaseError::RepositoryError(e.to_string()))
    }

    /// Sets or clears the color indicator of a folder.
    ///
    /// Loads all folders, finds the target by ID, applies the new color,
    /// and persists the updated list. Color validation (hex `#RRGGBB`)
    /// is performed by the domain entity itself.
    async fn set_folder_color(
        &self,
        id: FolderId,
        color: Option<String>,
    ) -> Result<(), UseCaseError> {
        let mut folders = self
            .config_repo
            .load_all()
            .await
            .map_err(|e| UseCaseError::RepositoryError(e.to_string()))?;

        let folder = folders
            .iter_mut()
            .find(|f| f.id == id)
            .ok_or_else(|| UseCaseError::FolderNotFound(id.to_string()))?;

        folder
            .set_color(color)
            .map_err(|e| UseCaseError::Domain(e.to_string()))?;

        self.config_repo
            .save_all(&folders)
            .await
            .map_err(|e| UseCaseError::RepositoryError(e.to_string()))
    }

    /// Sets or clears the parent folder for tree nesting.
    ///
    /// Validates that the parent exists, the move does not form a cycle,
    /// and the resulting nesting depth stays within `MAX_FOLDER_DEPTH`.
    async fn set_folder_parent(
        &self,
        id: FolderId,
        parent_id: Option<FolderId>,
    ) -> Result<(), UseCaseError> {
        let mut folders = self
            .config_repo
            .load_all()
            .await
            .map_err(|e| UseCaseError::RepositoryError(e.to_string()))?;

        if !folders.iter().any(|f| f.id == id) {
            return Err(UseCaseError::FolderNotFound(id.to_string()));
        }

        if let Some(parent) = &parent_id {
            if !folders.iter().any(|f| f.id == *parent) {
                return Err(UseCaseError::FolderNotFound(parent.to_string()));
            }
            // A move that would attach the folder to one of its own
            // descendants creates a cycle: walking up from the new parent
            // must never reach `id`. The walk is bounded so a corrupt
            // cycle in stored data cannot loop forever.
            let mut cursor: Option<&FolderId> = Some(parent);
            for _ in 0..MAX_FOLDER_DEPTH {
                match cursor {
                    Some(current) => {
                        if *current == id {
                            return Err(UseCaseError::Domain(DomainError::FolderCycle.to_string()));
                        }
                        cursor = folders
                            .iter()
                            .find(|f| f.id == *current)
                            .and_then(|f| f.parent_id.as_ref());
                    }
                    None => break,
                }
            }
            if cursor.is_some() {
                return Err(UseCaseError::Domain(DomainError::FolderCycle.to_string()));
            }
            // The resulting depth is the parent's level + 1 for the moved
            // folder itself plus the deepest nesting below it.
            if level_of(&folders, parent) + 1 + deepest_subtree_level(&folders, &id)
                > MAX_FOLDER_DEPTH
            {
                return Err(UseCaseError::Domain(
                    DomainError::FolderDepthExceeded.to_string(),
                ));
            }
        }

        let folder = folders
            .iter_mut()
            .find(|f| f.id == id)
            .ok_or_else(|| UseCaseError::FolderNotFound(id.to_string()))?;

        folder
            .set_parent(parent_id)
            .map_err(|e| UseCaseError::Domain(e.to_string()))?;

        self.config_repo
            .save_all(&folders)
            .await
            .map_err(|e| UseCaseError::RepositoryError(e.to_string()))
    }
}

impl ManageFoldersUseCase {
    /// Verifies that the parent exists and attaching a new leaf under it
    /// keeps the nesting depth within `MAX_FOLDER_DEPTH`.
    async fn validate_parent_exists_and_depth(
        &self,
        parent_id: &FolderId,
    ) -> Result<(), UseCaseError> {
        let folders = self
            .config_repo
            .load_all()
            .await
            .map_err(|e| UseCaseError::RepositoryError(e.to_string()))?;

        if !folders.iter().any(|f| f.id == *parent_id) {
            return Err(UseCaseError::FolderNotFound(parent_id.to_string()));
        }

        // A new child sits one level below its parent.
        if level_of(&folders, parent_id) + 1 > MAX_FOLDER_DEPTH {
            return Err(UseCaseError::Domain(
                DomainError::FolderDepthExceeded.to_string(),
            ));
        }
        Ok(())
    }
}

/// Returns the nesting level of `id` (1 for a root-level folder).
///
/// The walk up the parent chain is bounded to `MAX_FOLDER_DEPTH` so a
/// corrupt cycle in stored data cannot loop forever; a chain deeper than
/// the maximum reports `MAX_FOLDER_DEPTH + 1` so depth checks reject it.
fn level_of(folders: &[Folder], id: &FolderId) -> usize {
    let mut level = 1;
    let mut cursor: Option<&FolderId> = Some(id);
    while let Some(current) = cursor {
        if level > MAX_FOLDER_DEPTH {
            return MAX_FOLDER_DEPTH + 1;
        }
        level += 1;
        cursor = folders
            .iter()
            .find(|f| f.id == *current)
            .and_then(|f| f.parent_id.as_ref());
    }
    level - 1
}

/// Returns the deepest nesting level below `id` (0 for a leaf folder).
///
/// Traversal is bounded to `MAX_FOLDER_DEPTH` so a corrupt cycle in the
/// data cannot cause an infinite walk.
fn deepest_subtree_level(folders: &[Folder], id: &FolderId) -> usize {
    let mut deepest = 0;
    let mut stack: Vec<(&FolderId, usize)> = vec![(id, 0)];
    while let Some((node, level)) = stack.pop() {
        if level >= MAX_FOLDER_DEPTH {
            continue;
        }
        for child in folders
            .iter()
            .filter(|f| f.parent_id.as_ref() == Some(node))
        {
            stack.push((&child.id, level + 1));
        }
        if level > deepest {
            deepest = level;
        }
    }
    deepest
}
