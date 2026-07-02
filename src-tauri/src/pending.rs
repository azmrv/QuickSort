use parking_lot::Mutex;
use std::sync::OnceLock;

/// Хранит путь к файлу, который передан через --select-folder
static PENDING_FILE: OnceLock<Mutex<Option<String>>> = OnceLock::new();

pub fn get_pending_file() -> Option<String> {
    let lock = PENDING_FILE.get_or_init(|| Mutex::new(None));
    lock.lock().take()
}

pub fn set_pending_file(file: String) {
    let lock = PENDING_FILE.get_or_init(|| Mutex::new(None));
    *lock.lock() = Some(file);
}