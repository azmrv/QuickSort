//! Tauri adapter for progress reporting.
//!
//! Emits `operation-progress` events to the frontend during long-running
//! operations. Uses the same global handle pattern as `logging.rs`.

use std::sync::{Mutex, OnceLock};
use tauri::Emitter;

use quicksort_application::ports::outbound::{ProgressInfo, ProgressReporter};

static APP_HANDLE: OnceLock<Mutex<Option<tauri::AppHandle>>> = OnceLock::new();

pub fn set_app_handle(handle: tauri::AppHandle) {
    let _ = APP_HANDLE.set(Mutex::new(Some(handle)));
}

/// Tauri-based progress reporter that emits events to the frontend.
pub struct TauriProgressReporter;

impl TauriProgressReporter {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl ProgressReporter for TauriProgressReporter {
    async fn report(&self, progress: ProgressInfo) {
        if let Some(handle) = APP_HANDLE.get() {
            if let Ok(guard) = handle.lock() {
                if let Some(ref h) = *guard {
                    let _ = h.emit("operation-progress", &progress);
                }
            }
        }
    }
}
