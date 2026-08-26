//! Temporary storage for the file path(s) passed via the `--select-folder`
//! CLI flag or the IPC `SelectFolder` command.
//!
//! When the shell extension DLL invokes "📂 Все папки...", it sends a
//! `SelectFolder` command with one or more source file paths.  These paths
//! must be communicated to the React frontend so it can display the
//! `SelectorPage` instead of the normal editor.
//!
//! # Design Decision
//! A global `OnceLock<Mutex<Option<Vec<String>>>>` was chosen because the
//! CLI handler and the Tauri `AppState` are initialised at different points
//! in `main.rs`.  Once the application has fully moved to the Application
//! Facade, this global can be replaced by a field in `AppState`.
//!
//! # Future Work
//! - Move this into `AppState` to eliminate the global variable.

use parking_lot::Mutex;
use std::sync::OnceLock;

/// Global storage for file paths that should be opened in the Selector.
static PENDING_FILES: OnceLock<Mutex<Option<Vec<String>>>> = OnceLock::new();

/// Retrieves and clears the pending file paths, if any.
///
/// Returns a `Vec<String>` of all pending file paths.  The value is
/// returned exactly once — after that the storage is empty until a new
/// `SelectFolder` invocation.
pub fn get_pending_files() -> Vec<String> {
    let lock = PENDING_FILES.get_or_init(|| Mutex::new(None));
    lock.lock()
        .take()
        .unwrap_or_default()
        .into_iter()
        .map(sanitize_path)
        .collect()
}

/// Stores file paths that should be opened in the Selector.
///
/// Called from the CLI handler in `main.rs` or the IPC server when
/// a `SelectFolder` command is received.
pub fn set_pending_files(files: Vec<String>) {
    let lock = PENDING_FILES.get_or_init(|| Mutex::new(None));
    *lock.lock() = Some(files);
}

/// Retrieves and clears a single pending file path (backward compat).
///
/// Returns `Some(path)` if exactly one file is pending, `None` otherwise.
pub fn get_pending_file() -> Option<String> {
    let files = get_pending_files();
    if files.len() == 1 {
        Some(files.into_iter().next().unwrap())
    } else if files.is_empty() {
        None
    } else {
        // Multiple files — store them back so get_pending_files can return them.
        let lock = PENDING_FILES.get_or_init(|| Mutex::new(None));
        *lock.lock() = Some(files);
        None
    }
}

/// Stores a single file path (backward compat for CLI handler).
pub fn set_pending_file(file: String) {
    set_pending_files(vec![file]);
}

fn sanitize_path(path: String) -> String {
    path.chars()
        .filter(|c| !c.is_control())
        .collect::<String>()
        .trim()
        .to_string()
}
