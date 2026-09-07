//! IPC (Inter-Process Communication) module for the Tauri adapter.
//!
//! This module provides a platform-agnostic IPC server that receives
//! commands from the shell extension DLL (Windows) or Unix socket clients.
//! On Windows, it uses Named Pipes; on Linux/macOS, Unix Domain Sockets.
//! The framing protocol is defined in `quicksort-ipc-contract` and forwarded
//! decoded commands to the Application Facade.

use std::sync::{Arc, Mutex, OnceLock};
use tauri::AppHandle;

pub mod server;
pub mod transport;

#[cfg(target_os = "windows")]
pub mod framing;

#[cfg(target_os = "windows")]
pub mod named_pipe;

#[cfg(not(target_os = "windows"))]
pub mod unix_socket;

/// Global Tauri AppHandle stored during setup for use by the IPC server.
static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();

/// Stores the AppHandle so the IPC server can emit events and control windows.
///
/// Called once during Tauri setup. Subsequent calls are silently ignored.
pub fn set_app_handle(handle: AppHandle) {
    let _ = APP_HANDLE.set(handle);
}

/// Returns the stored AppHandle, if available.
pub(crate) fn get_app_handle() -> Option<&'static AppHandle> {
    APP_HANDLE.get()
}

/// Global lock serializing all background file operations (both the IPC
/// server's fire-and-forget operations and the persistent queue worker).
///
/// The JSON history repository is file-backed and not safe for concurrent
/// read-modify-write, so every execute path must hold this lock.
static OP_LOCK: OnceLock<Arc<Mutex<()>>> = OnceLock::new();

/// Returns the shared operation serialization lock, creating it on first use.
pub(crate) fn op_lock() -> Arc<Mutex<()>> {
    OP_LOCK.get_or_init(|| Arc::new(Mutex::new(()))).clone()
}
