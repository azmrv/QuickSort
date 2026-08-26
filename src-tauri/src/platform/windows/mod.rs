//! Windows-specific platform implementations.
//!
//! This module contains Windows-specific code for:
//! - COM Shell Extension registration
//! - DLL lifecycle management
//! - Registry operations

use std::path::PathBuf;

/// Get the path to the DLL next to the executable.
#[allow(dead_code)]
pub fn dll_path() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let dir = exe.parent()?;
    Some(dir.join("context_menu_dll.dll"))
}

/// Ensure the DLL is copied next to the executable.
///
/// In dev builds the DLL is in `target/{debug,release}/deps/`.
/// In installed builds it's already bundled via `tauri.conf.json` resources.
pub fn ensure_dll_copied() {
    let exe_dir = match std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
    {
        Some(d) => d,
        None => {
            tracing::warn!("Cannot determine exe directory");
            return;
        }
    };

    let dll = exe_dir.join("context_menu_dll.dll");
    if !dll.exists() {
        let deps_dll = exe_dir.join("deps").join("context_menu_dll.dll");
        if deps_dll.exists() {
            match std::fs::copy(&deps_dll, &dll) {
                Ok(_) => {
                    tracing::info!(src = %deps_dll.display(), dst = %dll.display(), "Copied DLL next to exe")
                }
                Err(e) => tracing::error!(error = %e, "Failed to copy DLL next to exe"),
            }
        } else {
            tracing::warn!(
                path = %dll.display(),
                "DLL not found — COM registration will fail until DLL is built"
            );
        }
    } else {
        tracing::debug!(path = %dll.display(), "DLL found next to exe");
    }

    // Copy quicksort.ico next to exe so the shell extension DLL can find it.
    let icon_dest = exe_dir.join("quicksort.ico");
    if !icon_dest.exists() {
        if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
            let icon_src = std::path::PathBuf::from(manifest_dir)
                .join("..")
                .join("resources")
                .join("quicksort.ico");
            if icon_src.exists() {
                match std::fs::copy(&icon_src, &icon_dest) {
                    Ok(_) => {
                        tracing::info!(src = %icon_src.display(), dst = %icon_dest.display(), "Copied icon next to exe")
                    }
                    Err(e) => tracing::warn!(error = %e, "Failed to copy icon next to exe"),
                }
            }
        }
        if !icon_dest.exists() {
            let icon_cwd = std::path::Path::new("resources").join("quicksort.ico");
            if icon_cwd.exists() {
                match std::fs::copy(&icon_cwd, &icon_dest) {
                    Ok(_) => {
                        tracing::info!(src = %icon_cwd.display(), dst = %icon_dest.display(), "Copied icon next to exe (cwd fallback)")
                    }
                    Err(e) => {
                        tracing::warn!(error = %e, "Failed to copy icon next to exe (cwd fallback)")
                    }
                }
            }
        }
    }
}

/// Write the owner PID file.
///
/// The PID file is used by the DLL to determine if the main process is still running.
pub fn write_owner_pid() {
    let pid = std::process::id();
    let config_dir = super::paths::config_dir();
    let _ = std::fs::create_dir_all(&config_dir);
    let pid_path = config_dir.join("dll_owner.pid");
    match std::fs::write(&pid_path, pid.to_string()) {
        Ok(()) => tracing::info!(pid, path = %pid_path.display(), "owner PID written"),
        Err(e) => tracing::error!(error = %e, "failed to write owner PID"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dll_path_returns_valid_path() {
        let path = dll_path();
        assert!(path.is_some());
        assert!(path.unwrap().ends_with("context_menu_dll.dll"));
    }
}
