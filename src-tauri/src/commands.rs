use tauri::State;
use parking_lot::Mutex;
use crate::folder::repository::JsonRepository;
use crate::folder::service::FolderService;
use crate::models::{Folder, FolderId};
use crate::context_menu::model::MenuModel;
use crate::context_menu::registry::RegistryInstaller;

pub struct AppState {
    pub service: FolderService<JsonRepository>,
    pub exe_path: Mutex<String>,
}

#[tauri::command]
pub fn get_folders(state: State<AppState>) -> Vec<Folder> {
    state.service.list().unwrap_or_default()
}

#[tauri::command]
pub fn update_folders(state: State<AppState>, folders: Vec<Folder>) -> Result<(), String> {
    state.service.update_all(folders.clone()).map_err(|e| e.to_string())?;
    let exe_path = state.exe_path.lock().clone();
    if folders.is_empty() {
        RegistryInstaller::uninstall().map_err(|e: anyhow::Error| e.to_string())?;
    } else {
        let model = MenuModel::from_folders(&folders);
        RegistryInstaller::install(&model, &exe_path).map_err(|e: anyhow::Error| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn toggle_favorite(state: State<AppState>, id: FolderId) -> Result<(), String> {
    state.service.toggle_favorite(id).map_err(|e| e.to_string())?;
    let exe_path = state.exe_path.lock().clone();
    let folders = state.service.list().map_err(|e| e.to_string())?;
    let model = MenuModel::from_folders(&folders);
    RegistryInstaller::install(&model, &exe_path).map_err(|e: anyhow::Error| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_mode() -> String {
    "Editor".to_string()
}

#[tauri::command]
pub fn move_file(src: String, dest_dir: String) -> Result<String, String> {
    crate::move_engine::MoveEngine::move_file(
        std::path::Path::new(&src),
        std::path::Path::new(&dest_dir),
    )
        .map(|p| p.to_string_lossy().into())
        .map_err(|e| e.to_string())
}