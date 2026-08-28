//! Plugin system implementations.
//!
//! This module contains adapters for various plugin types:
//! - WCX: Total Commander packer plugins (archives)

// WCX plugins are Windows DLLs loaded via LoadLibraryW/GetProcAddress.
// The adapter is Windows-only; its FFI types (winapi, OsStrExt) do not
// exist on other platforms.
#[cfg(target_os = "windows")]
pub mod wcx_adapter;

#[cfg(target_os = "windows")]
pub use wcx_adapter::{WcxPluginAdapter, WcxPluginLoader};
