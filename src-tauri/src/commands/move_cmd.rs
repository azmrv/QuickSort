use chrono::Utc;
use tauri::State;
use crate::state::{AppState, LogEntry};
use crate::activity_log;

#[tauri::command]
pub fn move_file(state: State<AppState>, src: String, dest_dir: String) -> Result<String, String> {
    let result = crate::move_engine::MoveEngine::move_file(
        std::path::Path::new(&src),
        std::path::Path::new(&dest_dir),
    );
    let status = match &result {
        Ok(_) => "Успех".to_string(),
        Err(e) => format!("Ошибка: {}", e),
    };
    state.logs.lock().push(LogEntry {
        timestamp: Utc::now().to_rfc3339(),
        event: format!("Перемещение {} → {}", src, dest_dir),
        status,
    });
    let status = match &result {
        Ok(_) => "Успех".into(),
        Err(e) => format!("Ошибка: {}", e),
    };
    activity_log::add_log(
        &state.logs,
        format!("Перемещение {} → {}", src, dest_dir),
        status,
    );
    result.map(|p| p.to_string_lossy().into()).map_err(|e| e.to_string())
}





