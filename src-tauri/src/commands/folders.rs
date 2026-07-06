use tauri::State;
use crate::state::{AppState, LogEntry};
use crate::models::{Folder, FolderId};
use crate::activity_log;
use chrono::Utc;

#[tauri::command]
pub fn get_folders(state: State<AppState>) -> Vec<Folder> {
    state.service.list().unwrap_or_default()
}

#[tauri::command]
pub fn update_folders(state: State<AppState>, folders: Vec<Folder>) -> Result<(), String> {
    // Сохраняем конфигурацию через сервис
    state.service.update_all(folders.clone())
        .map_err(|e: anyhow::Error| e.to_string())?;

    // Логируем успех
    let entry = LogEntry {
        timestamp: Utc::now().to_rfc3339(),
        event: format!("Обновлён список папок ({} шт.)", folders.len()),
        status: "Успех".to_string(),
    };
    // state.logs.lock().push(entry.clone());
    activity_log::add_log(&state.logs, entry.event, entry.status);

    Ok(())
}

#[tauri::command]
pub fn toggle_favorite(state: State<AppState>, id: FolderId) -> Result<(), String> {
    state.service.toggle_favorite(id)
        .map_err(|e: anyhow::Error| e.to_string())?;

    // Логируем
    let entry = LogEntry {
        timestamp: Utc::now().to_rfc3339(),
        event: format!("Переключено избранное для папки {}", id.0),
        status: "Успех".to_string(),
    };
    // state.logs.lock().push(entry.clone());
    activity_log::add_log(&state.logs, entry.event, entry.status);

    Ok(())
}