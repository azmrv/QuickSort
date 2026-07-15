//! Tauri command wrappers over ApplicationFacade.
//!
//! This module defines the public API that the React frontend (and CLI)
//! use to interact with the application.  Every command is a thin adapter
//! that:
//! 1. Receives data from the frontend (often as plain strings).
//! 2. Converts it into domain types or DTOs.
//! 3. Delegates to the `ApplicationFacade`.
//! 4. Maps errors into plain strings suitable for user display.
//!
//! # Design Decision
//! All commands accept and return simple types (`String`, `Folder`, etc.)
//! rather than the internal `UseCaseError` because Tauri serialises
//! everything as JSON and the frontend expects human-readable messages.

use tauri::State;

use quicksort_domain::{Folder, FolderId, OperationId};
use quicksort_application::dtos::{OperationCommand, OperationResult};
use quicksort_application::ApplicationFacade;

// ---------------------------------------------------------------------------
// Execute Operation
// ---------------------------------------------------------------------------

/// Execute a file operation (Move, Copy, Delete, Rename).
///
/// The frontend constructs an `OperationCommand` and sends it here.
/// The facade runs it through the full pipeline (validation, conflict
/// resolution, execution) and returns the result.
#[tauri::command]
pub async fn execute_operation(
    state: State<'_, ApplicationFacade>,
    command: OperationCommand,
) -> Result<OperationResult, String> {
    state
        .execute_operation(command)
        .await
        .map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// Undo Operation
// ---------------------------------------------------------------------------

/// Undo a previously executed operation.
///
/// The frontend passes the operation ID (as a UUID string).  We validate
/// it and convert it into an `OperationId` before calling the facade.
#[tauri::command]
pub async fn undo_operation(
    state: State<'_, ApplicationFacade>,
    operation_id: String,
) -> Result<OperationResult, String> {
    // Validate the UUID string before passing it to the domain.
    let id = OperationId::from_string(&operation_id)
        .map_err(|e| format!("Invalid operation ID: {}", e))?;
    state
        .undo_operation(id)
        .await
        .map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// Folder CRUD
// ---------------------------------------------------------------------------

/// Return all saved folders.
#[tauri::command]
pub async fn get_folders(
    state: State<'_, ApplicationFacade>,
) -> Result<Vec<Folder>, String> {
    state
        .get_folders()
        .await
        .map_err(|e| e.to_string())
}

/// Add a new folder to the list.
#[tauri::command]
pub async fn add_folder(
    state: State<'_, ApplicationFacade>,
    folder: Folder,
) -> Result<(), String> {
    state
        .add_folder(folder)
        .await
        .map_err(|e| e.to_string())
}

/// Remove a folder by its UUID.
#[tauri::command]
pub async fn remove_folder(
    state: State<'_, ApplicationFacade>,
    id: String,
) -> Result<(), String> {
    let folder_id = FolderId::from_string(&id)
        .map_err(|e| format!("Invalid folder ID: {}", e))?;
    state
        .remove_folder(folder_id)
        .await
        .map_err(|e| e.to_string())
}

/// Rename a folder (change its display name).
#[tauri::command]
pub async fn rename_folder(
    state: State<'_, ApplicationFacade>,
    id: String,
    new_name: String,
) -> Result<(), String> {
    let folder_id = FolderId::from_string(&id)
        .map_err(|e| format!("Invalid folder ID: {}", e))?;
    state
        .rename_folder(folder_id, new_name)
        .await
        .map_err(|e| e.to_string())
}