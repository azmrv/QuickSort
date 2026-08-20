use crate::state::AppState;
use quicksort_application::{
    ExecuteOperation, Folder, FolderId, GetFolders, ManageFolders, OperationId, UndoOperation,
    WindowsPath,
};
use tauri::State;

#[tauri::command]
pub async fn get_folders_v2(state: State<'_, AppState>) -> Result<Vec<Folder>, String> {
    state.facade.get_all().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_folder_v2(
    state: State<'_, AppState>,
    name: String,
    path: String,
) -> Result<(), String> {
    let windows_path = WindowsPath::new(&path).map_err(|e| format!("Invalid path: {}", e))?;
    let folder = Folder::new(&name, windows_path).map_err(|e| format!("Invalid folder: {}", e))?;
    state
        .facade
        .add_folder(folder)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_folder_v2(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let folder_id = FolderId::from_string(&id).map_err(|e| format!("Invalid folder ID: {}", e))?;
    state
        .facade
        .remove_folder(folder_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn toggle_favorite_v2(
    state: State<'_, AppState>,
    id: String,
    order: Option<u32>,
) -> Result<(), String> {
    let _ = order;
    let folder_id = FolderId::from_string(&id).map_err(|e| format!("Invalid folder ID: {}", e))?;
    state
        .facade
        .toggle_favorite(folder_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn execute_operation_v2(
    state: State<'_, AppState>,
    command: quicksort_application::OperationCommand,
) -> Result<quicksort_application::OperationResult, String> {
    state
        .facade
        .execute(command)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn undo_operation_v2(
    state: State<'_, AppState>,
    operation_id: String,
) -> Result<quicksort_application::OperationResult, String> {
    let id = OperationId::from_string(&operation_id)
        .map_err(|e| format!("Invalid operation ID: {}", e))?;
    state.facade.undo(id).await.map_err(|e| e.to_string())
}

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
    use winreg::enums::*;
    use winreg::RegKey;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    hkcu.open_subkey(r"Software\Classes\*\shellex\ContextMenuHandlers\QuickSort")
        .is_ok()
}

#[tauri::command]
pub fn get_logs() -> Vec<serde_json::Value> {
    Vec::new()
}

#[tauri::command]
pub fn register_com_server() -> Result<String, String> {
    use winreg::enums::*;
    use winreg::RegKey;

    if !is_user_admin() {
        return Err("Administrator privileges required to register COM server".to_string());
    }

    let appdata = std::env::var("APPDATA")
        .map_err(|e| format!("APPDATA environment variable not set: {}", e))?;
    let dll_path = std::path::PathBuf::from(&appdata)
        .join("QuickSort")
        .join("context_menu_dll.dll");
    let dll_path_str = dll_path.to_string_lossy().to_string();

    let clsid = "{12345678-1234-1234-1234-1234567890AB}";
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    let (clsid_key, _) = hkcu
        .create_subkey(format!("Software\\Classes\\CLSID\\{}", clsid))
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

    std::process::Command::new("cmd")
        .args(&["/C", "taskkill /f /im explorer.exe && start explorer.exe"])
        .spawn()
        .map_err(|e| format!("Failed to restart Explorer: {}", e))?;

    Ok("COM server registered successfully. Explorer has been restarted.".to_string())
}

#[tauri::command]
pub fn unregister_com_server() -> Result<String, String> {
    use winreg::enums::*;
    use winreg::RegKey;

    let clsid = "{12345678-1234-1234-1234-1234567890AB}";
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

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

    let clsid_path = format!("Software\\Classes\\CLSID\\{}", clsid);
    if let Ok(key) = hkcu.open_subkey(&clsid_path) {
        key.delete_subkey_all("")
            .map_err(|e| format!("Failed to delete CLSID key '{}': {}", clsid_path, e))?;
    }

    std::process::Command::new("cmd")
        .args(&["/C", "taskkill /f /im explorer.exe && start explorer.exe"])
        .spawn()
        .map_err(|e| format!("Failed to restart Explorer: {}", e))?;

    Ok("COM server unregistered successfully.".to_string())
}

fn is_user_admin() -> bool {
    std::process::Command::new("net")
        .args(["session"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
