use tauri::State;
use crate::state::AppState;
use crate::models::{Folder, FolderId};
use crate::state::LogEntry;
use chrono::Utc;
use crate::activity_log;
use crate::folder::repository::JsonRepository;
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::SW_HIDE;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

#[tauri::command]
pub fn get_folders(state: State<AppState>) -> Vec<Folder> {
    state.service.list().unwrap_or_default()
}

#[tauri::command]
pub fn update_folders(state: State<AppState>, folders: Vec<Folder>) -> Result<(), String> {
    state.service.update_all(folders.clone()).map_err(|e: anyhow::Error| e.to_string())?;

    let exe_path = state.exe_path.lock().clone();

    let repo = JsonRepository::new().map_err(|e| e.to_string())?;
    let config_path = repo.config_path().to_string_lossy().to_string();

    let operation: Vec<u16> = OsStr::new("runas").encode_wide().chain(Some(0)).collect();
    let file: Vec<u16> = OsStr::new(&state.admin_exe_path).encode_wide().chain(Some(0)).collect();
    let params: Vec<u16> = format!("install \"{}\" \"{}\"", config_path, exe_path)
        .encode_utf16()
        .chain(Some(0))
        .collect();
    let directory: Vec<u16> = OsStr::new("").encode_wide().chain(Some(0)).collect();

    unsafe {
        let result = ShellExecuteW(
            None,
            windows::core::PCWSTR(operation.as_ptr()),
            windows::core::PCWSTR(file.as_ptr()),
            windows::core::PCWSTR(params.as_ptr()),
            windows::core::PCWSTR(directory.as_ptr()),
            SW_HIDE,
        );
        if result.0 as i32 <= 32 {
            return Err(format!("Ошибка запуска admin-клиента: код {}", result.0 as isize));
        }
    }

    state.logs.lock().push(LogEntry {
        timestamp: Utc::now().to_rfc3339(),
        event: format!("Обновление списка папок ({} шт.)", folders.len()),
        status: "Успех".to_string(),
    });
    activity_log::add_log(
        &state.logs,
        format!("Обновлён список папок ({} шт.)", folders.len()),
        "Успех".into(),
    );

    Ok(())
}


#[tauri::command]
pub fn toggle_favorite(state: State<AppState>, id: FolderId) -> Result<(), String> {
    state.service.toggle_favorite(id).map_err(|e: anyhow::Error| e.to_string())?;
    let exe_path = state.exe_path.lock().clone();
    let _folders = state.service.list().map_err(|e: anyhow::Error| e.to_string())?;

    let repo = JsonRepository::new().map_err(|e| e.to_string())?;
    let config_path = repo.config_path().to_string_lossy().to_string();

    let operation: Vec<u16> = OsStr::new("runas").encode_wide().chain(Some(0)).collect();
    let file: Vec<u16> = OsStr::new(&state.admin_exe_path).encode_wide().chain(Some(0)).collect();
    let params: Vec<u16> = format!("install \"{}\" \"{}\"", config_path, exe_path)
        .encode_utf16()
        .chain(Some(0))
        .collect();
    let directory: Vec<u16> = OsStr::new("").encode_wide().chain(Some(0)).collect();

    unsafe {
        let result = ShellExecuteW(
            None,
            windows::core::PCWSTR(operation.as_ptr()),
            windows::core::PCWSTR(file.as_ptr()),
            windows::core::PCWSTR(params.as_ptr()),
            windows::core::PCWSTR(directory.as_ptr()),
            SW_HIDE,
        );
        if result.0 as i32 <= 32 {
            return Err(format!("Ошибка запуска admin-клиента: код {}", result.0 as isize));
        }
    }
    Ok(())
}