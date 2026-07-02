use crate::move_engine::MoveEngine;

#[tauri::command]
pub fn move_file(src: String, dest_dir: String) -> Result<String, String> {
    let src = std::path::Path::new(&src);
    let dest = std::path::Path::new(&dest_dir);
    MoveEngine::move_file(src, dest)
        .map(|p| p.to_string_lossy().into())
        .map_err(|e| e.to_string())
}