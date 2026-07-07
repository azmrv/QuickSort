use chrono::Utc;
use tauri::State;
use winreg::enums::*;
use winreg::RegKey;
use crate::activity_log;
use crate::state::{AppState, LogEntry};

#[tauri::command]
pub fn get_mode() -> String {
    "Editor".to_string()
}

#[tauri::command]
pub fn get_pending_file() -> Option<String> {
    crate::pending::get_pending_file()
}

#[tauri::command]
pub fn check_menu_status() -> bool {
    let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
    hkcu.open_subkey(r"Software\Classes\*\shellex\ContextMenuHandlers\QuickSort").is_ok()
}

#[tauri::command]
pub fn get_logs(state: State<AppState>) -> Vec<LogEntry> {
    state.logs.lock().clone()
}

#[tauri::command]
pub fn register_com_server(state: State<AppState>) -> Result<String, String> {
    // Единый путь к DLL в %APPDATA%\QuickSort
    let appdata = std::env::var("APPDATA").map_err(|e| e.to_string())?;
    let dll_path = std::path::PathBuf::from(&appdata)
        .join("QuickSort")
        .join("context_menu_dll.dll");
    let dll_path_str = dll_path.to_string_lossy().to_string();

    let clsid = "{12345678-1234-1234-1234-1234567890AB}";
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    // 1. Регистрируем CLSID
    let (clsid_key, _) = hkcu
        .create_subkey(format!("Software\\Classes\\CLSID\\{}", clsid))
        .map_err(|e| format!("Не удалось создать CLSID: {}", e))?;
    clsid_key
        .set_value("", &"QuickSort Context Menu Extension")
        .map_err(|e| format!("Не удалось задать имя: {}", e))?;

    // 2. Прописываем путь к DLL
    let (inproc, _) = clsid_key
        .create_subkey("InprocServer32")
        .map_err(|e| format!("Не удалось создать InprocServer32: {}", e))?;
    inproc
        .set_value("", &dll_path_str)
        .map_err(|e| format!("Не удалось задать путь DLL: {}", e))?;
    inproc
        .set_value("ThreadingModel", &"Apartment")
        .map_err(|e| format!("Не удалось задать ThreadingModel: {}", e))?;

    // 3. Добавляем обработчики для файлов и папок
    let handlers = ["*", "Directory", "Directory\\Background", "Drive"];
    for handler in &handlers {
        let path = format!(
            "Software\\Classes\\{}\\shellex\\ContextMenuHandlers\\QuickSort",
            handler
        );
        let (key, _) = hkcu
            .create_subkey(&path)
            .map_err(|e| format!("Не удалось создать {}: {}", path, e))?;
        key.set_value("", &clsid)
            .map_err(|e| format!("Не удалось задать CLSID для {}: {}", handler, e))?;
    }

    // 4. Логирование
    let entry = LogEntry {
        timestamp: Utc::now().to_rfc3339(),
        event: "COM-сервер зарегистрирован".into(),
        status: "Успех".into(),
    };
    activity_log::add_log(&state.logs, entry.event, entry.status);

    // 5. Автоматический перезапуск Проводника
    std::process::Command::new("cmd")
        .args(&["/C", "taskkill /f /im explorer.exe && start explorer.exe"])
        .spawn()
        .map_err(|e| format!("Не удалось перезапустить Проводник: {}", e))?;

    Ok("COM-сервер успешно зарегистрирован и Проводник перезапущен.".to_string())
}

#[tauri::command]
pub fn unregister_com_server(state: State<AppState>) -> Result<String, String> {
    let clsid = "{12345678-1234-1234-1234-1234567890AB}";
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    // Удаляем обработчики
    for handler in &["*", "Directory", "Directory\\Background", "Drive"] {
        let path = format!(
            "Software\\Classes\\{}\\shellex\\ContextMenuHandlers\\QuickSort",
            handler
        );
        if let Ok(key) = hkcu.open_subkey(&path) {
            key.delete_subkey_all("")
                .map_err(|e| format!("Не удалось удалить {}: {}", path, e))?;
        }
    }

    // Удаляем CLSID
    let clsid_path = format!("Software\\Classes\\CLSID\\{}", clsid);
    if let Ok(key) = hkcu.open_subkey(&clsid_path) {
        key.delete_subkey_all("")
            .map_err(|e| format!("Не удалось удалить {}: {}", clsid_path, e))?;
    }

    // Логирование
    let entry = LogEntry {
        timestamp: Utc::now().to_rfc3339(),
        event: "COM-сервер удалён".into(),
        status: "Успех".into(),
    };
    activity_log::add_log(&state.logs, entry.event, entry.status);

    Ok("COM-сервер успешно удалён из реестра.".to_string())
}

#[tauri::command]
pub async fn undo_operation(state: State<'_, AppState>) -> Result<(), String> {
    state.facade.undo_last().await.map_err(|e| e.to_string())
}