//! Inbound port for managing folders (add, remove, rename, reorder).
//!
//! This port defines the contract that adapters (Tauri, CLI, IPC) use to
//! perform CRUD operations on the folder configuration.
//! It is implemented by `ManageFoldersUseCase`.

use async_trait::async_trait;
use crate::errors::UseCaseError;
use quicksort_domain::{Folder, FolderId};

/// Inbound port for modifying the folder configuration.
///
/// # Role in the Architecture
/// - **Inbound port** – called by outer layers (adapters).
/// - **Implemented by** – `ManageFoldersUseCase` (Application Layer).
/// - **Used by** – Tauri commands, IPC server, any UI that modifies folder data.
///
/// # Error Handling
/// All methods return `UseCaseError` to insulate callers from infrastructure
/// details. Common error variants include:
/// - `RepositoryError` – the underlying storage failed.
/// - `FolderNotFound` – the specified folder does not exist.
/// - `InvalidCommand` – the input data is malformed.
#[async_trait]
pub trait ManageFolders: Send + Sync {
    /// Adds a new folder to the configuration.
    ///
    /// The folder must have a unique `FolderId`; duplicates are prevented
    /// by the repository implementation.
    async fn add_folder(&self, folder: Folder) -> Result<(), UseCaseError>;

    /// Removes a folder by its unique identifier.
    ///
    /// If the folder does not exist, the operation is idempotent and
    /// does not return an error (the repository simply ignores the call).
    async fn remove_folder(&self, id: FolderId) -> Result<(), UseCaseError>;

    /// Renames an existing folder.
    ///
    /// The folder is identified by `id`. If the folder is not found,
    /// a `FolderNotFound` error is returned.
    async fn rename_folder(&self, id: FolderId, new_name: String) -> Result<(), UseCaseError>;

    /// Toggles the favorite status of a folder.
    ///
    /// If the folder is currently a favorite, it becomes non-favorite,
    /// and vice versa. The method does **not** modify the sort order;
    /// use a dedicated `reorder_folders` method (planned) for that purpose.
    ///
    /// # Note
    /// The `order` parameter has been removed from this method because
    /// toggling a favorite is a boolean operation unrelated to ordering.
    /// Order management will be handled separately when the SQLite
    /// migration adds a dedicated `sort_order` field.
    // OLD: async fn toggle_favorite(&self, id: FolderId, order: i32) -> Result<(), UseCaseError>;
    // NEW: order parameter removed – see TASK-015 for ordering feature
    async fn toggle_favorite(&self, id: FolderId) -> Result<(), UseCaseError>;
}