use crate::state::LogEntry;
use chrono::Utc;
use parking_lot::Mutex;
use std::path::PathBuf;

/// Путь к файлу лога
fn log_path() -> PathBuf {
    let appdata = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
    let mut path = PathBuf::from(appdata);
    path.push("QuickSort");
    std::fs::create_dir_all(&path).ok();
    path.join("activity.json")
}

/// Загрузить логи из файла (максимум 500)
pub fn load_logs() -> Vec<LogEntry> {
    let path = log_path();
    if path.exists() {
        let data = std::fs::read_to_string(&path).unwrap_or_default();
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        vec![]
    }
}

/// Сохранить логи в файл, обрезав до последних 500
pub fn save_logs(logs: &[LogEntry]) {
    let path = log_path();
    let trimmed: Vec<&LogEntry> = logs.iter().rev().take(500).rev().collect();
    if let Ok(json) = serde_json::to_string_pretty(&trimmed) {
        std::fs::write(&path, json).ok();
    }
}

/// Добавить запись в лог (используется из команд)
pub fn add_log(logs: &Mutex<Vec<LogEntry>>, event: String, status: String) {
    let entry = LogEntry {
        timestamp: Utc::now().to_rfc3339(),
        event,
        status,
    };
    let mut guard = logs.lock();
    guard.push(entry);
    // Сохраняем после каждого добавления (можно оптимизировать, но для 500 записей норм)
    save_logs(&guard);
}