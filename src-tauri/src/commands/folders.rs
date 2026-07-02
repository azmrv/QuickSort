use tauri::State;
use crate::state::AppState;
use crate::models::{Folder, FolderId};
use crate::context_menu::model::MenuModel;
use crate::context_menu::registry::RegistryInstaller;

#[tauri::command]
pub fn get_folders(state: State<AppState>) -> Vec<Folder> {
    state.service.list().unwrap_or_default()
}

#[tauri::command]
pub fn update_folders(state: State<AppState>, folders: Vec<Folder>) -> Result<(), String> {
    state.service.update_all(folders.clone()).map_err(|e: anyhow::Error| e.to_string())?;
    let exe_path = state.exe_path.lock().clone();
    let model = MenuModel::from_folders(&folders);
    RegistryInstaller::install(&model, &exe_path).map_err(|e: anyhow::Error| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn toggle_favorite(state: State<AppState>, id: FolderId) -> Result<(), String> {
    state.service.toggle_favorite(id).map_err(|e: anyhow::Error| e.to_string())?;
    let exe_path = state.exe_path.lock().clone();
    let folders = state.service.list().map_err(|e: anyhow::Error| e.to_string())?;
    let model = MenuModel::from_folders(&folders);
    RegistryInstaller::install(&model, &exe_path).map_err(|e: anyhow::Error| e.to_string())?;
    Ok(())
}