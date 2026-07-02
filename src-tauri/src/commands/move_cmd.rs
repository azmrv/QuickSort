#[tauri::command]
pub fn move_file(src: String, dest_dir: String) -> Result<String, String> {
    crate::move_engine::MoveEngine::move_file(
        std::path::Path::new(&src),
        std::path::Path::new(&dest_dir),
    )
        .map(|p| p.to_string_lossy().into())
        .map_err(|e| e.to_string())
}