fn main() {
    // Build the Tauri application (generate icons, resources, etc.)
    tauri_build::build();

    // After building the main executable, copy the context menu DLL
    // to the user's AppData folder so the COM registration can find it.
    // This ensures that after `cargo build` (or `npm run tauri build`),
    // the DLL is already in place and the user can register it with one click.
    copy_dll_to_appdata();
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

    // Determine the source path of the DLL.
    // In a cargo workspace the DLL is output to the shared `target/` dir at the
    // workspace root, NOT to `context-menu-dll/target/`.  We also keep the old
    // path as a fallback for non-workspace (standalone) builds.
    let workspace_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .to_path_buf();

    let candidates = [
        // Workspace build (preferred): repo/target/release/
        workspace_root
            .join("target")
            .join("release")
            .join("context_menu_dll.dll"),
        // Standalone build fallback: repo/context-menu-dll/target/release/
        workspace_root
            .join("context-menu-dll")
            .join("target")
            .join("release")
            .join("context_menu_dll.dll"),
    ];

    let dll_src = candidates.iter().find(|p| p.exists()).cloned();

    let dll_src = match dll_src {
        Some(path) => path,
        None => {
            println!(
                "cargo:warning=DLL not found in {} – skipping copy (build the DLL first)",
                workspace_root.display()
            );
            return;
        }
    };

    // Copy the DLL to the destination
    let dll_dest = appdata.join("context_menu_dll.dll");
    match std::fs::copy(&dll_src, &dll_dest) {
        Ok(_) => println!("cargo:warning=DLL copied to {}", dll_dest.display()),
        Err(e) => println!(
            "cargo:warning=Failed to copy DLL from {} to {}: {}",
            dll_src.display(),
            dll_dest.display(),
            e
        ),
    }

    // Copy icon file next to DLL for context menu icon
    let icon_src = workspace_root
        .join("context-menu-dll")
        .join("quicksort.ico");
    if icon_src.exists() {
        let icon_dest = appdata.join("quicksort.ico");
        if let Err(e) = std::fs::copy(&icon_src, &icon_dest) {
            println!(
                "cargo:warning=Failed to copy icon to {}: {}",
                icon_dest.display(),
                e
            );
        }
    }
}
