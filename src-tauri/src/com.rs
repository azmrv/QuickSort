use std::path::PathBuf;
use winreg::enums::*;
use winreg::RegKey;

const CLSID: &str = "{12345678-1234-1234-1234-1234567890AB}";
const HANDLERS: &[&str] = &["*", "Directory", "Directory\\Background", "Drive"];

pub fn is_registered() -> bool {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    hkcu.open_subkey(format!("Software\\Classes\\CLSID\\{}", CLSID))
        .is_ok()
}

pub fn dll_path() -> Option<PathBuf> {
    let appdata = std::env::var("APPDATA").ok()?;
    Some(
        PathBuf::from(appdata)
            .join("QuickSort")
            .join("context_menu_dll.dll"),
    )
}

pub fn register() -> Result<(), String> {
    let dll = dll_path().ok_or("APPDATA not set")?;
    if !dll.exists() {
        return Err(format!("DLL not found: {}", dll.display()));
    }
    let dll_str = dll.to_string_lossy().to_string();

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    let (clsid_key, _) = hkcu
        .create_subkey(format!("Software\\Classes\\CLSID\\{}", CLSID))
        .map_err(|e| format!("create CLSID key: {}", e))?;
    clsid_key
        .set_value("", &"QuickSort Context Menu Extension")
        .map_err(|e| format!("set CLSID name: {}", e))?;

    let (inproc, _) = clsid_key
        .create_subkey("InprocServer32")
        .map_err(|e| format!("create InprocServer32: {}", e))?;
    inproc
        .set_value("", &dll_str)
        .map_err(|e| format!("set DLL path: {}", e))?;
    inproc
        .set_value("ThreadingModel", &"Apartment")
        .map_err(|e| format!("set ThreadingModel: {}", e))?;

    for handler in HANDLERS {
        let path = format!(
            "Software\\Classes\\{}\\shellex\\ContextMenuHandlers\\QuickSort",
            handler
        );
        let (key, _) = hkcu
            .create_subkey(&path)
            .map_err(|e| format!("create handler '{}': {}", path, e))?;
        key.set_value("", &CLSID)
            .map_err(|e| format!("set CLSID for '{}': {}", handler, e))?;
    }

    restart_explorer();
    Ok(())
}

pub fn unregister() -> Result<(), String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    for handler in HANDLERS {
        let path = format!(
            "Software\\Classes\\{}\\shellex\\ContextMenuHandlers\\QuickSort",
            handler
        );
        if let Ok(key) = hkcu.open_subkey(&path) {
            key.delete_subkey_all("")
                .map_err(|e| format!("delete handler '{}': {}", path, e))?;
        }
    }

    let clsid_path = format!("Software\\Classes\\CLSID\\{}", CLSID);
    if let Ok(key) = hkcu.open_subkey(&clsid_path) {
        key.delete_subkey_all("")
            .map_err(|e| format!("delete CLSID: {}", e))?;
    }

    restart_explorer();
    Ok(())
}

fn restart_explorer() {
    let _ = std::process::Command::new("cmd")
        .args(["/C", "taskkill /f /im explorer.exe && start explorer.exe"])
        .spawn();
}
