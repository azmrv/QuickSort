use crate::config::{save_folders, TargetFolder};
use crate::context_menu::ContextMenuBuilder;
use std::sync::Mutex;
use tauri::State;

pub struct AppState {
    pub folders: Mutex<Vec<TargetFolder>>,
    pub exe_path: String,
}

#[tauri::command]
pub fn get_folders(state: State<AppState>) -> Vec<TargetFolder> {
    state.folders.lock().unwrap().clone()
}

#[tauri::command]
pub fn update_folders(
    state: State<AppState>,
    folders: Vec<TargetFolder>,
) -> Result<(), String> {
    // Сохраняем JSON
    save_folders(&folders).map_err(|e| e.to_string())?;
    // Обновляем контекстное меню через удобный строитель
    // ContextMenuBuilder::new(&state.exe_path, &folders)
    //     .build()
    //     .map_err(|e| e.to_string())?;
    ContextMenuBuilder::new(&state.exe_path, &folders)
        .build()
        .map_err(|e| {
            eprintln!("Ошибка контекстного меню: {:?}", e);
            e.to_string()
        })?;
    // Обновляем кэш в памяти
    *state.folders.lock().map_err(|e| e.to_string())? = folders;
    Ok(())
}

#[tauri::command]
pub fn check_menu_status() -> bool {
    crate::context_menu::is_menu_registered()
}