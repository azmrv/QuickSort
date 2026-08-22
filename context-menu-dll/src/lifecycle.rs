//! DLL lifecycle: owner-process liveness check.
//!
//! The DLL is loaded by Explorer.exe, not by our app. The app writes its PID
//! to `%APPDATA%/QuickSort/dll_owner.pid` on startup. The DLL reads that file
//! and checks whether the process is still alive.
//!
//! If the owner is dead the DLL refuses to create new COM objects
//! (`DllGetClassObject` → `CLASS_E_CLASSNOTAVAILABLE`) and tells COM it may
//! unload (`DllCanUnloadNow` → `S_OK`). Explorer will eventually release the
//! DLL.
//!
//! Emergency kill-switch: the `unregister_com_server` Tauri command removes
//! all registry entries and restarts Explorer, which force-unloads the DLL.

use std::fs;
use std::path::PathBuf;

use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::Threading::{GetExitCodeProcess, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};

fn pid_file_path() -> Option<PathBuf> {
    let appdata = std::env::var("APPDATA").ok()?;
    Some(PathBuf::from(appdata).join("QuickSort").join("dll_owner.pid"))
}

fn read_owner_pid() -> Option<u32> {
    let content = fs::read_to_string(pid_file_path()?).ok()?;
    content.trim().parse().ok()
}

fn is_process_alive(pid: u32) -> bool {
    unsafe {
        let handle: HANDLE = match OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
            Ok(h) => h,
            Err(_) => return false,
        };
        let mut exit_code = 0u32;
        let ok = GetExitCodeProcess(handle, &mut exit_code);
        let _ = CloseHandle(handle);
        match ok {
            Ok(()) => exit_code == 259, // STILL_ACTIVE
            Err(_) => false,
        }
    }
}

/// Returns `true` when the Tauri app that owns this DLL is still running.
pub fn is_owner_alive() -> bool {
    let pid = match read_owner_pid() {
        Some(p) => p,
        None => return false,
    };
    is_process_alive(pid)
}

/// Returns `true` when COM may unload this DLL.
///
/// Either there are no instances, or the owner process is dead.
pub fn can_unload(instance_count: u32) -> bool {
    if instance_count == 0 {
        return true;
    }
    !is_owner_alive()
}
