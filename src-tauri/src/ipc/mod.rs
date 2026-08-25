//! IPC (Inter-Process Communication) module for the Tauri adapter.
//!
//! This module contains the Named Pipe server that receives commands from
//! the shell extension DLL.  It uses the framing protocol defined in
//! `quicksort-ipc-contract` and forwards decoded commands to the
//! Application Facade.

use std::sync::OnceLock;
use tauri::AppHandle;

pub mod framing;
pub mod server;

/// Global Tauri AppHandle stored during setup for use by the pipe server.
static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();

/// Stores the AppHandle so the pipe server can emit events and control windows.
///
/// Called once during Tauri setup. Subsequent calls are silently ignored.
pub fn set_app_handle(handle: AppHandle) {
    let _ = APP_HANDLE.set(handle);
}

/// Returns the stored AppHandle, if available.
pub(crate) fn get_app_handle() -> Option<&'static AppHandle> {
    APP_HANDLE.get()
}
