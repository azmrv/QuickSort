//! Platform abstraction layer.
//!
//! This module provides platform-specific implementations for:
//! - Shell integration (context menu, quick actions)
//! - IPC transport (Named Pipe on Windows, Unix Socket on Linux/macOS)
//! - Path validation and config directory resolution

#[cfg(target_os = "windows")]
pub mod windows;

pub mod paths;
