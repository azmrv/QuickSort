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
    hkcu.open_subkey(r"Software\Classes\*\shell\QuickSort").is_ok()
}