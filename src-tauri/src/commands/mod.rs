use crate::state::AppState;
use quicksort_application::{
    ExecuteOperation, Folder, FolderId, GetFolders, ManageFolders, OperationId, UndoOperation,
    WindowsPath,
};
use tauri::State;

#[tauri::command]
pub async fn get_folders_v2(state: State<'_, AppState>) -> Result<Vec<Folder>, String> {
    tracing::info!(command = "get_folders_v2", "handling");
    let result = state.facade.get_all().await.map_err(|e| e.to_string());
    match &result {
        Ok(folders) => tracing::info!(command = "get_folders_v2", count = folders.len(), "OK"),
        Err(e) => tracing::error!(command = "get_folders_v2", error = %e, "FAIL"),
    }
    result
}

#[tauri::command]
pub async fn add_folder_v2(
    state: State<'_, AppState>,
    name: String,
    path: String,
) -> Result<(), String> {
    tracing::info!(command = "add_folder_v2", name = %name, path = %path, "handling");
    let windows_path = WindowsPath::new(&path).map_err(|e| {
        tracing::error!(command = "add_folder_v2", error = %e, "invalid path");
        format!("Invalid path: {}", e)
    })?;
    let folder = Folder::new(&name, windows_path).map_err(|e| {
        tracing::error!(command = "add_folder_v2", error = %e, "invalid folder");
        format!("Invalid folder: {}", e)
    })?;
    let result = state
        .facade
        .add_folder(folder)
        .await
        .map_err(|e| e.to_string());
    match &result {
        Ok(()) => tracing::info!(command = "add_folder_v2", "OK"),
        Err(e) => tracing::error!(command = "add_folder_v2", error = %e, "FAIL"),
    }
    result
}

#[tauri::command]
pub async fn remove_folder_v2(state: State<'_, AppState>, id: String) -> Result<(), String> {
    tracing::info!(command = "remove_folder_v2", id = %id, "handling");
    let folder_id = FolderId::from_string(&id).map_err(|e| {
        tracing::error!(command = "remove_folder_v2", error = %e, "invalid folder ID");
        format!("Invalid folder ID: {}", e)
    })?;
    let result = state
        .facade
        .remove_folder(folder_id)
        .await
        .map_err(|e| e.to_string());
    match &result {
        Ok(()) => tracing::info!(command = "remove_folder_v2", "OK"),
        Err(e) => tracing::error!(command = "remove_folder_v2", error = %e, "FAIL"),
    }
    result
}

#[tauri::command]
pub async fn toggle_favorite_v2(
    state: State<'_, AppState>,
    id: String,
    order: Option<u32>,
) -> Result<(), String> {
    tracing::info!(command = "toggle_favorite_v2", id = %id, order = ?order, "handling");
    let _ = order;
    let folder_id = FolderId::from_string(&id).map_err(|e| {
        tracing::error!(command = "toggle_favorite_v2", error = %e, "invalid folder ID");
        format!("Invalid folder ID: {}", e)
    })?;
    let result = state
        .facade
        .toggle_favorite(folder_id)
        .await
        .map_err(|e| e.to_string());
    match &result {
        Ok(()) => tracing::info!(command = "toggle_favorite_v2", "OK"),
        Err(e) => tracing::error!(command = "toggle_favorite_v2", error = %e, "FAIL"),
    }
    result
}

#[tauri::command]
pub async fn execute_operation_v2(
    state: State<'_, AppState>,
    command: quicksort_application::OperationCommand,
) -> Result<quicksort_application::OperationResult, String> {
    tracing::info!(command = "execute_operation_v2", op_type = ?command.operation_type, sources = ?command.source_paths, "handling");
    let result = state
        .facade
        .execute(command)
        .await
        .map_err(|e| e.to_string());
    match &result {
        Ok(r) => tracing::info!(command = "execute_operation_v2", state = ?r.state, files = r.processed_files, "OK"),
        Err(e) => tracing::error!(command = "execute_operation_v2", error = %e, "FAIL"),
    }
    result
}

#[tauri::command]
pub async fn undo_operation_v2(
    state: State<'_, AppState>,
    operation_id: String,
) -> Result<quicksort_application::OperationResult, String> {
    tracing::info!(command = "undo_operation_v2", operation_id = %operation_id, "handling");
    let id = OperationId::from_string(&operation_id).map_err(|e| {
        tracing::error!(command = "undo_operation_v2", error = %e, "invalid operation ID");
        format!("Invalid operation ID: {}", e)
    })?;
    let result = state.facade.undo(id).await.map_err(|e| e.to_string());
    match &result {
        Ok(r) => tracing::info!(command = "undo_operation_v2", state = ?r.state, "OK"),
        Err(e) => tracing::error!(command = "undo_operation_v2", error = %e, "FAIL"),
    }
    result
}

#[tauri::command]
pub fn get_mode() -> String {
    tracing::debug!(command = "get_mode", "handling");
    "Editor".to_string()
}

#[tauri::command]
pub fn get_pending_file() -> Option<String> {
    tracing::info!(command = "get_pending_file", "handling");
    let file = crate::pending::get_pending_file();
    tracing::info!(command = "get_pending_file", file = ?file, "OK");
    file
}

#[tauri::command]
pub fn check_menu_status() -> bool {
    tracing::debug!(command = "check_menu_status", "handling");
    crate::com::is_registered()
}

#[tauri::command]
pub fn get_logs() -> Vec<serde_json::Value> {
    tracing::debug!(command = "get_logs", "handling — returning empty (stub)");
    Vec::new()
}

#[tauri::command]
pub fn register_com_server() -> Result<String, String> {
    tracing::info!(command = "register_com_server", "handling");
    crate::com::register()?;
    tracing::info!(command = "register_com_server", "OK — Explorer restarted");
    Ok("COM server registered successfully. Explorer has been restarted.".to_string())
}

#[tauri::command]
pub fn unregister_com_server() -> Result<String, String> {
    tracing::info!(command = "unregister_com_server", "handling");
    crate::com::unregister()?;
    tracing::info!(command = "unregister_com_server", "OK — Explorer restarted");
    Ok("COM server unregistered successfully.".to_string())
}
