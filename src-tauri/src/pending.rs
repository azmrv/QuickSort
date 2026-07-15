//! Temporary storage for the file path passed via the `--select-folder` CLI flag.
//!
//! When the shell extension DLL invokes "📂 Все папки...", it launches
//! `quicksort.exe select-folder <path>`.  That file path must be communicated
//! to the React frontend so it can display the `SelectorPage` instead of the
//! normal editor.
//!
//! # Design Decision
//! A global `OnceLock<Mutex<Option<String>>>` was chosen because the CLI
//! handler and the Tauri `AppState` are initialised at different points in
//! `main.rs`.  Once the application has fully moved to the Application Facade,
//! this global can be replaced by a field in `AppState` (e.g.,
//! `pending_file: Mutex<Option<String>>`).
//!
//! # Future Work
//! - Move this into `AppState` to eliminate the global variable.
//! - Pass the file path directly to the React frontend via a Tauri event
//!   instead of a polling command.

use parking_lot::Mutex;
use std::sync::OnceLock;

/// Global storage for a file path that should be opened in the Selector.
// OLD: Хранит путь к файлу, который передан через --select-folder
static PENDING_FILE: OnceLock<Mutex<Option<String>>> = OnceLock::new();

/// Retrieves and clears the pending file path, if any.
///
/// This is called by the `get_pending_file` Tauri command when the
/// React frontend starts.  The value is returned exactly once – after
/// that the storage is empty until a new `--select-folder` invocation.
pub fn get_pending_file() -> Option<String> {
    let lock = PENDING_FILE.get_or_init(|| Mutex::new(None));
    lock.lock().take()
}

/// Stores a file path that should be opened in the Selector.
///
/// Called from the CLI handler in `main.rs` when the app is started
/// with the `select-folder` subcommand.
pub fn set_pending_file(file: String) {
    let lock = PENDING_FILE.get_or_init(|| Mutex::new(None));
    *lock.lock() = Some(file);
}