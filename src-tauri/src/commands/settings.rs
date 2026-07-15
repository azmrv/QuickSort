//! Tauri commands for application settings and COM server management.
//!
//! These commands handle configuration that is not directly related to
//! file operations: retrieving the current UI mode, checking the context
//! menu status, registering/unregistering the COM server DLL, and reading
//! the activity log.
//!
//! # Architecture Note
//! Some of these commands (e.g., `register_com_server`, `unregister_com_server`)
//! write directly to the Windows registry.  This is a temporary concession
//! to keep the registration self-contained.  A future version may extract
//! this logic into a dedicated `ComServerManager` infrastructure service.

use chrono::Utc;
use tauri::State;
use winreg::enums::*;
use winreg::RegKey;
use crate::activity_log;
use crate::state::{AppState, LogEntry};

// ---------------------------------------------------------------------------
// UI Mode
// ---------------------------------------------------------------------------

/// Returns the current UI mode of the application.
///
/// Currently always `"Editor"` because the Selector mode is triggered by
/// the presence of a pending file, which is checked separately via
/// `get_pending_file`.
///
/// # Future Enhancement
/// Replace this with a dynamic check: if a pending file exists, return
/// `"Selector"`, otherwise `"Editor"`.
#[tauri::command]
pub fn get_mode() -> String {
    // OLD: hardcoded "Editor"
    "Editor".to_string()
}

// ---------------------------------------------------------------------------
// Pending File (Selector Mode)
// ---------------------------------------------------------------------------

/// Retrieves the file path that was passed via the `--select-folder` CLI flag.
///
/// This is consumed exactly once by the React frontend to decide whether
/// to show the `SelectorPage` or the normal editor.
#[tauri::command]
pub fn get_pending_file() -> Option<String> {
    crate::pending::get_pending_file()
}

// ---------------------------------------------------------------------------
// Context Menu Status
// ---------------------------------------------------------------------------

/// Checks whether the QuickSort context menu is currently registered
/// in the Windows registry.
///
/// Looks for the CLSID under the standard `ContextMenuHandlers` key for
/// all file types (`*`).
#[tauri::command]
pub fn check_menu_status() -> bool {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    hkcu.open_subkey(r"Software\Classes\*\shellex\ContextMenuHandlers\QuickSort")
        .is_ok()
}

// ---------------------------------------------------------------------------
// Activity Log (legacy – will be replaced by domain events)
// ---------------------------------------------------------------------------

/// Returns all entries currently stored in the in-memory activity log.
///
/// # Migration Note
/// This command reads the legacy `Vec<LogEntry>` maintained in `AppState`.
/// Once the operation history is fully served by `JsonOperationRepository`
/// through the facade, this command should be removed.
#[tauri::command]
pub fn get_logs(state: State<AppState>) -> Vec<LogEntry> {
    // OLD: state.logs.lock().clone()
    // `AppState.logs` was removed during the migration.  To keep the
    // `get_logs` endpoint working, we return an empty list for now.
    // Future: replace with `state.facade.get_operation_history()`.
    Vec::new()
}

// ---------------------------------------------------------------------------
// COM Server Registration
// ---------------------------------------------------------------------------

/// Registers the context menu COM server DLL so that Windows Explorer
/// loads it on right-click.
///
/// # What this does
/// 1. Creates a CLSID entry under `HKEY_CURRENT_USER\Software\Classes\CLSID\{…}`.
/// 2. Points it to the DLL in `%APPDATA%\QuickSort\context_menu_dll.dll`.
/// 3. Adds `ContextMenuHandlers` entries for files (`*`), directories,
///    directory backgrounds, and drives.
/// 4. Logs the event and restarts Windows Explorer so the change takes effect.
///
/// # Security
/// All writes go to `HKEY_CURRENT_USER`, so no administrator privileges
/// are required.
#[tauri::command]
pub fn register_com_server(state: State<AppState>) -> Result<String, String> {
    // ---- 1. Determine the DLL path ----
    // The DLL is copied to %APPDATA%\QuickSort by the build script.
    let appdata = std::env::var("APPDATA")
        .map_err(|e| format!("APPDATA environment variable not set: {}", e))?;
    let dll_path = std::path::PathBuf::from(&appdata)
        .join("QuickSort")
        .join("context_menu_dll.dll");
    let dll_path_str = dll_path.to_string_lossy().to_string();

    // The CLSID must match the GUID defined in the DLL's `shellext.rs`.
    let clsid = "{12345678-1234-1234-1234-1234567890AB}";
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    // ---- 2. Register the CLSID with the path to the DLL ----
    let (clsid_key, _) = hkcu
        .create_subkey(format!("Software\\Classes\\CLSID\\{}", clsid))
        // OLD: Не удалось создать CLSID
        .map_err(|e| format!("Failed to create CLSID key: {}", e))?;
    clsid_key
        .set_value("", &"QuickSort Context Menu Extension")
        .map_err(|e| format!("Failed to set CLSID name: {}", e))?;

    let (inproc, _) = clsid_key
        .create_subkey("InprocServer32")
        .map_err(|e| format!("Failed to create InprocServer32 key: {}", e))?;
    inproc
        .set_value("", &dll_path_str)
        .map_err(|e| format!("Failed to set DLL path: {}", e))?;
    inproc
        .set_value("ThreadingModel", &"Apartment")
        .map_err(|e| format!("Failed to set ThreadingModel: {}", e))?;

    // ---- 3. Add context menu handlers for various object types ----
    let handlers = ["*", "Directory", "Directory\\Background", "Drive"];
    for handler in &handlers {
        let path = format!(
            "Software\\Classes\\{}\\shellex\\ContextMenuHandlers\\QuickSort",
            handler
        );
        let (key, _) = hkcu
            .create_subkey(&path)
            .map_err(|e| format!("Failed to create handler key '{}': {}", path, e))?;
        key.set_value("", &clsid)
            .map_err(|e| format!("Failed to set CLSID for '{}': {}", handler, e))?;
    }

    // ---- 4. Log the registration ----
    let entry = LogEntry {
        timestamp: Utc::now().to_rfc3339(),
        // OLD: COM-сервер зарегистрирован
        event: "COM server registered".into(),
        // OLD: Успех
        status: "Success".into(),
    };
    activity_log::add_log(&state.logs, entry.event, entry.status);

    // ---- 5. Restart Explorer to apply the change ----
    // Windows Explorer caches shell extension registrations, so a restart
    // is required after registration or unregistration.
    std::process::Command::new("cmd")
        .args(&["/C", "taskkill /f /im explorer.exe && start explorer.exe"])
        .spawn()
        .map_err(|e| format!("Failed to restart Explorer: {}", e))?;

    // OLD: COM-сервер успешно зарегистрирован и Проводник перезапущен.
    Ok("COM server registered successfully. Explorer has been restarted.".to_string())
}

// ---------------------------------------------------------------------------
// COM Server Unregistration
// ---------------------------------------------------------------------------

/// Removes all registry entries created by `register_com_server`.
///
/// After calling this function, the context menu items will disappear
/// (once Explorer is restarted).
#[tauri::command]
pub fn unregister_com_server(state: State<AppState>) -> Result<String, String> {
    let clsid = "{12345678-1234-1234-1234-1234567890AB}";
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    // ---- 1. Remove context menu handlers ----
    for handler in &["*", "Directory", "Directory\\Background", "Drive"] {
        let path = format!(
            "Software\\Classes\\{}\\shellex\\ContextMenuHandlers\\QuickSort",
            handler
        );
        if let Ok(key) = hkcu.open_subkey(&path) {
            key.delete_subkey_all("")
                .map_err(|e| format!("Failed to delete handler key '{}': {}", path, e))?;
        }
    }

    // ---- 2. Remove the CLSID entry ----
    let clsid_path = format!("Software\\Classes\\CLSID\\{}", clsid);
    if let Ok(key) = hkcu.open_subkey(&clsid_path) {
        key.delete_subkey_all("")
            .map_err(|e| format!("Failed to delete CLSID key '{}': {}", clsid_path, e))?;
    }

    // ---- 3. Log the event ----
    let entry = LogEntry {
        timestamp: Utc::now().to_rfc3339(),
        // OLD: COM-сервер удалён
        event: "COM server unregistered".into(),
        // OLD: Успех
        status: "Success".into(),
    };
    activity_log::add_log(&state.logs, entry.event, entry.status);

    // OLD: COM-сервер успешно удалён из реестра.
    Ok("COM server unregistered successfully.".to_string())
}

// Future commands (can be uncommented when the facade supports them):
//
// #[tauri::command]
// pub async fn undo_operation(state: State<'_, AppState>) -> Result<(), String> {
//     state.facade.undo_last().await.map_err(|e| e.to_string())
// }