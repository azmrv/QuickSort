fn main() {
    tauri_build::build();

    // Копируем DLL из context-menu-dll в папку с конфигурацией пользователя
    let appdata = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
    let dest_dir = std::path::PathBuf::from(&appdata).join("QuickSort");
    std::fs::create_dir_all(&dest_dir).ok();

    let dll_src = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("context-menu-dll")
        .join("target")
        .join("release")
        .join("context_menu_dll.dll");

    if dll_src.exists() {
        let dll_dest = dest_dir.join("context_menu_dll.dll");
        std::fs::copy(&dll_src, &dll_dest).ok();
        println!("cargo:warning=DLL copied to {}", dll_dest.display());
    }
}