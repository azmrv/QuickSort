//! Activity log persistence – **DEPRECATED**.
//!
//! This module provided ad-hoc logging directly from Tauri commands.
//! It is being replaced by the domain events mechanism:
//!   - `Operation` aggregate records `DomainEvent`s (OperationStarted,
//!     OperationCompleted, etc.)
//!   - `JsonOperationRepository` persists operations (which contain events)
//!   - The `LogPage` in the frontend reads the operation history via
//!     the `get_logs` Tauri command.
//!
//! Once the Tauri commands are updated to read logs from
//! `JsonOperationRepository` (through the Application Facade), this module
//! can be deleted.
//!
//! # Migration Path
//! 1. Replace `add_log` calls with domain events raised by Use Cases.
//! 2. Replace `get_logs` Tauri command with a facade call that reads
//!    operations from `JsonOperationRepository`.
//! 3. Delete this file.

// OLD: use crate::state::LogEntry;
// OLD: use chrono::Utc;
// OLD: use parking_lot::Mutex;
// OLD: use std::path::PathBuf;

// All functions below are kept for backward compatibility during the
// transition.  They will be removed once the migration is complete.

/// Returns the path to the activity log JSON file.
fn log_path() -> PathBuf {
    let appdata = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
    let mut path = PathBuf::from(appdata);
    path.push("QuickSort");
    std::fs::create_dir_all(&path).ok();
    path.join("activity.json")
}

/// Loads logs from the JSON file (keeps at most the last 500 entries).
pub fn load_logs() -> Vec<crate::state::LogEntry> {
    let path = log_path();
    if path.exists() {
        let data = std::fs::read_to_string(&path).unwrap_or_default();
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        vec![]
    }
}

/// Saves logs to the JSON file, trimming to the last 500 entries.
pub fn save_logs(logs: &[crate::state::LogEntry]) {
    let path = log_path();
    let trimmed: Vec<&crate::state::LogEntry> = logs.iter().rev().take(500).rev().collect();
    if let Ok(json) = serde_json::to_string_pretty(&trimmed) {
        std::fs::write(&path, json).ok();
    }
}

/// Appends a log entry and persists the updated list.
pub fn add_log(logs: &parking_lot::Mutex<Vec<crate::state::LogEntry>>, event: String, status: String) {
    let entry = crate::state::LogEntry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        event,
        status,
    };
    let mut guard = logs.lock();
    guard.push(entry);
    // Saving after every addition is acceptable for small volumes (≤500 entries).
    save_logs(&guard);
}