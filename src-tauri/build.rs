fn main() {
    // Build the Tauri application (generate icons, resources, etc.)
    tauri_build::build();

    // After building the main executable, copy the context menu DLL
    // to the user's AppData folder so the COM registration can find it.
    // This ensures that after `cargo build` (or `npm run tauri build`),
    // the DLL is already in place and the user can register it with one click.
    copy_dll_to_appdata();
}

/// Removes stale `.old` DLL files left by the rename-and-copy strategy.
fn cleanup_stale_old(dir: &std::path::Path) {
    let old = dir.join("context_menu_dll.dll.old");
    if old.exists() {
        let _ = std::fs::remove_file(&old);
    }
}

/// Searches for `context_menu_dll.dll` in the given directory.
/// Returns the full path if found, `None` otherwise.
fn find_dll(dir: &std::path::Path) -> Option<std::path::PathBuf> {
    let path = dir.join("context_menu_dll.dll");
    if path.exists() {
        Some(path)
    } else {
        None
    }
}

/// Copies `context_menu_dll.dll` from the workspace output to `%APPDATA%\QuickSort\`.
///
/// # Why we do this at build time
/// The COM server DLL must be placed in a known, stable location so that
/// `register_com_server` can write a registry entry pointing to it.
/// `%APPDATA%\QuickSort` was chosen because it is writable without
/// administrator privileges and persists across application updates.
///
/// # Panics
/// This function logs warnings on failure but never panics – a missing DLL
/// at build time is not a hard error (it may have been built separately).
fn copy_dll_to_appdata() {
    // Determine the destination directory: %APPDATA%\QuickSort
    let appdata = match std::env::var("APPDATA") {
        Ok(path) => std::path::PathBuf::from(path).join("QuickSort"),
        Err(_) => {
            // APPDATA is not set – fall back to the current directory.
            // This can happen in CI environments or when running as a different user.
            println!("cargo:warning=APPDATA not set – DLL will be copied to current directory");
            std::path::PathBuf::from("QuickSort")
        }
    };
    // Ensure the destination directory exists
    if let Err(e) = std::fs::create_dir_all(&appdata) {
        println!(
            "cargo:warning=Failed to create directory {}: {}",
            appdata.display(),
            e
        );
        return;
    }

    // Determine the source path of the DLL — check multiple locations:
    // 1. context-menu-dll/target/release/ (standalone build)
    // 2. context-menu-dll/target/debug/ (standalone build)
    // 3. workspace/target/release/ (workspace build)
    // 4. workspace/target/debug/ (workspace build)
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir.parent().unwrap_or(manifest_dir.as_path());
    let dll_base = workspace_root.join("context-menu-dll").join("target");
    let workspace_target = workspace_root.join("target");

    let dll_src = if let Some(p) = find_dll(&dll_base.join("release")) {
        p
    } else if let Some(p) = find_dll(&dll_base.join("debug")) {
        p
    } else if let Some(p) = find_dll(&workspace_target.join("release")) {
        p
    } else if let Some(p) = find_dll(&workspace_target.join("debug")) {
        p
    } else {
        println!(
            "cargo:warning=DLL not found — skipping copy (build context-menu-dll first)"
        );
        return;
    };

    // Verify that the source DLL exists before attempting to copy
    if !dll_src.exists() {
        println!(
            "cargo:warning=DLL not found at {} – skipping copy (build the DLL first)",
            dll_src.display()
        );
        return;
    }

    // Copy the DLL to the destination.
    // If the target is locked (Explorer has it loaded), rename the old
    // DLL to `.old` first, then copy the new one. Windows allows renaming
    // a loaded file; the old handle keeps working under the old name.
    let dll_dest = appdata.join("context_menu_dll.dll");
    match std::fs::copy(&dll_src, &dll_dest) {
        Ok(_) => {
            println!("cargo:warning=DLL copied to {}", dll_dest.display());
            cleanup_stale_old(&appdata);
        }
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied
            || e.raw_os_error() == Some(32) =>
        {
            // File is locked — rename old -> .old, then copy fresh
            let old = appdata.join("context_menu_dll.dll.old");
            let _ = std::fs::remove_file(&old);
            if std::fs::rename(&dll_dest, &old).is_ok() {
                match std::fs::copy(&dll_src, &dll_dest) {
                    Ok(_) => println!(
                        "cargo:warning=DLL copied (renamed locked file to .old) to {}",
                        dll_dest.display()
                    ),
                    Err(e2) => println!(
                        "cargo:warning=Failed to copy DLL after rename: {}",
                        e2
                    ),
                }
            } else {
                println!(
                    "cargo:warning=Failed to copy DLL from {} to {}: {} (rename also failed)",
                    dll_src.display(),
                    dll_dest.display(),
                    e
                );
            }
        }
        Err(e) => println!(
            "cargo:warning=Failed to copy DLL from {} to {}: {}",
            dll_src.display(),
            dll_dest.display(),
            e
        ),
    }
}
