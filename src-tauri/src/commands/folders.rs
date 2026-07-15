//! Tauri commands for folder management.
//!
//! These commands are thin wrappers around the Application Facade.
//! They receive requests from the React frontend, delegate to the
//! appropriate Use Case, and return results (or errors).
//!
//! # Migration note
//! The old `update_folders` command that saved the entire folder list
//! at once has been replaced by individual `add_folder`, `remove_folder`,
//! and `rename_folder` commands.  This gives the frontend finer control
//! and avoids overwriting concurrent changes.

use tauri::State;
use crate::state::AppState;
use quicksort_domain::{Folder, FolderId};
use quicksort_application::errors::UseCaseError;

/// Returns all folders currently stored in the configuration.
///
/// The frontend calls this on startup and after any modification to
/// refresh the folder list.
#[tauri::command]
pub async fn get_folders(
    state: State<'_, AppState>,
) -> Result<Vec<Folder>, String> {
    state.facade.get_folders().await.map_err(|e| e.to_string())
}

/// Adds a new folder to the configuration.
///
/// The folder must contain a unique ID.  Duplicate IDs are rejected
/// by the use case.
#[tauri::command]
pub async fn add_folder(
    state: State<'_, AppState>,
    folder: Folder,
) -> Result<(), String> {
    state.facade.add_folder(folder).await.map_err(|e| e.to_string())
}

/// Removes a folder by its UUID.
///
/// This is idempotent – removing a non-existent folder is not an error.
#[tauri::command]
pub async fn remove_folder(
    state: State<'_, AppState>,
    id: FolderId,
) -> Result<(), String> {
    state.facade.remove_folder(id).await.map_err(|e| e.to_string())
}

/// Renames a folder (changes its display name).
#[tauri::command]
pub async fn rename_folder(
    state: State<'_, AppState>,
    folder_id: FolderId,
    new_name: String,
) -> Result<(), String> {
    state.facade.rename_folder(folder_id, new_name).await.map_err(|e| e.to_string())
}

/// Toggles the favourite status of a folder.
///
/// Favourite folders appear directly in the context menu for quick access.
#[tauri::command]
pub async fn toggle_favorite(
    state: State<'_, AppState>,
    id: FolderId,
) -> Result<(), String> {
    state.facade.toggle_favorite(id).await.map_err(|e| e.to_string())
}