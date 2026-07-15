//! Tauri command wrappers over ApplicationFacade.

use tauri::State;

use quicksort_domain::{Folder, FolderId, OperationId};
use quicksort_application::dtos::{OperationCommand, OperationResult};
use quicksort_application::ApplicationFacade;

/// Execute an operation (move, copy, delete, rename)
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

/// Undo a previously executed operation
#[tauri::command]
pub async fn undo_operation(
    state: State<'_, ApplicationFacade>,
    operation_id: String,
) -> Result<OperationResult, String> {
    let id = OperationId::from_string(&operation_id)
        .map_err(|e| format!("Invalid operation ID: {}", e))?;
    state
        .undo_operation(id)
        .await
        .map_err(|e| e.to_string())
}

/// Get all saved folders
#[tauri::command]
pub async fn get_folders(
    state: State<'_, ApplicationFacade>,
) -> Result<Vec<Folder>, String> {
    state
        .get_folders()
        .await
        .map_err(|e| e.to_string())
}

/// Add a new folder to the list
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

/// Remove a folder from the list by ID
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

/// Rename a folder by ID
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