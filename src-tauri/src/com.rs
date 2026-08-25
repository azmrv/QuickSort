use std::path::PathBuf;
use std::process::Command;
use winreg::enums::*;
use winreg::RegKey;

const CLSID: &str = "{12345678-1234-1234-1234-1234567890AB}";
const HANDLERS: &[&str] = &["AllFilesystemObjects", "Directory", "Drive"];

/// Registration status returned by [`check_registration`].
pub enum RegistrationStatus {
    /// Not registered — CLSID key or handler keys missing.
    NotRegistered,
    /// Registered but the DLL path in registry does not match the actual DLL
    /// on disk (e.g. DLL was updated or moved).
    PathMismatch { stored: String, expected: String },
    /// Fully registered: CLSID key exists, DLL path matches, DLL file present.
    Active,
    /// DLL file does not exist on disk.
    DllMissing,
}

impl std::fmt::Display for RegistrationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotRegistered => write!(f, "not registered"),
            Self::PathMismatch { stored, expected } => {
                write!(f, "path mismatch: stored={}, expected={}", stored, expected)
            }
            Self::Active => write!(f, "active"),
            Self::DllMissing => write!(f, "DLL missing"),
        }
    }
}

/// Perform a full registration check.
pub fn check_registration() -> RegistrationStatus {
    let dll = match dll_path() {
        Some(p) => p,
        None => return RegistrationStatus::DllMissing,
    };
    if !dll.exists() {
        return RegistrationStatus::DllMissing;
    }

    let expected_path = dll.to_string_lossy().to_string();

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let clsid_path = format!("Software\\Classes\\CLSID\\{}", CLSID);

    // Check CLSID key
    let clsid_key = match hkcu.open_subkey(&clsid_path) {
        Ok(k) => k,
        Err(_) => return RegistrationStatus::NotRegistered,
    };

    // Check InprocServer32
    let inproc = match clsid_key.open_subkey("InprocServer32") {
        Ok(k) => k,
        Err(_) => return RegistrationStatus::NotRegistered,
    };
    let stored: String = inproc.get_value("").unwrap_or_default();

    if stored != expected_path {
        return RegistrationStatus::PathMismatch {
            stored,
            expected: expected_path,
        };
    }

    // Check at least one handler key exists
    let mut any_handler = false;
    for handler in HANDLERS {
        let handler_path = format!(
            "Software\\Classes\\{}\\shellex\\ContextMenuHandlers\\QuickSort",
            handler
        );
        if hkcu.open_subkey(&handler_path).is_ok() {
            any_handler = true;
            break;
        }
    }
    if !any_handler {
        return RegistrationStatus::NotRegistered;
    }

    RegistrationStatus::Active
}

/// Simple check: is the CLSID key present, DLL exists and path matches?
pub fn is_registered() -> bool {
    matches!(check_registration(), RegistrationStatus::Active)
}

pub fn dll_path() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let dir = exe.parent()?;
    Some(dir.join("context_menu_dll.dll"))
}

fn write_registry_keys() -> Result<(), String> {
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

    Ok(())
}

/// Register COM server keys. Explorer picks up the handler on next context menu invocation.
pub fn register() -> Result<(), String> {
    // Remove stale handler keys from previous versions that are no longer in HANDLERS.
    // `*` was removed because Windows file-type ProgIDs override it.
    // `Directory\Background` was removed because it shows the menu on desktop right-click.
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let stale_handlers = ["*", "Directory\\Background"];
    for handler in &stale_handlers {
        let stale_path = format!(
            "Software\\Classes\\{}\\shellex\\ContextMenuHandlers\\QuickSort",
            handler
        );
        if let Ok(parent) = hkcu.open_subkey_with_flags(
            format!(
                "Software\\Classes\\{}\\shellex\\ContextMenuHandlers",
                handler
            ),
            KEY_ALL_ACCESS,
        ) {
            if parent.delete_subkey("QuickSort").is_ok() {
                tracing::info!("Removed stale handler key: {}", stale_path);
            }
        }
    }

    write_registry_keys()?;
    Ok(())
}

pub fn unregister() -> Result<(), String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    // Delete handler keys so Explorer won't load this extension anymore.
    for handler in HANDLERS {
        let parent_path = format!(
            "Software\\Classes\\{}\\shellex\\ContextMenuHandlers",
            handler
        );
        if let Ok(parent) = hkcu.open_subkey_with_flags(&parent_path, KEY_ALL_ACCESS) {
            if let Err(e) = parent.delete_subkey("QuickSort") {
                tracing::warn!("delete 'QuickSort' from '{}': {}", parent_path, e);
            } else {
                tracing::info!("deleted handler for '{}'", handler);
            }
        }
    }

    // Delete CLSID key.
    if let Ok(parent) = hkcu.open_subkey_with_flags("Software\\Classes\\CLSID", KEY_ALL_ACCESS) {
        if let Err(e) = parent.delete_subkey(CLSID) {
            tracing::warn!("delete CLSID key: {}", e);
        } else {
            tracing::info!("deleted CLSID key");
        }
    }

    // Restart Explorer so it unloads the DLL from memory.
    restart_explorer();

    Ok(())
}

/// Restart Explorer so it re-reads COM handler registrations from the registry.
fn restart_explorer() {
    tracing::info!("Restarting Explorer to pick up new COM registration");
    let _ = Command::new("taskkill")
        .args(["/f", "/im", "explorer.exe"])
        .output();
    std::thread::sleep(std::time::Duration::from_millis(500));
    let _ = Command::new("explorer.exe").spawn();
}
