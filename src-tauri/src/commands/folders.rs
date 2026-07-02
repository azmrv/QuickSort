use tauri::State;
use crate::state::AppState;
use crate::models::Folder;

#[tauri::command]
pub fn get_folders(state: State<AppState>) -> Vec<Folder> {
    state.service.list().unwrap_or_default()
}

#[tauri::command]
pub fn update_folders(state: State<AppState>, folders: Vec<Folder>) -> Result<(), String> {
    state.service.update_all(folders).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn toggle_favorite(state: State<AppState>, id: uuid::Uuid) -> Result<(), String> {
    let folder_id = crate::models::FolderId(id);
    state.service.toggle_favorite(folder_id).map_err(|e| e.to_string())
}