//! Use cases (application business logic orchestration).
//!
//! This module contains the concrete implementations of all inbound ports.
//! Each use case coordinates domain entities and outbound ports to fulfill
//! a specific business requirement (e.g., executing an operation, undoing it,
//! managing folders).
//!
//! # Module Organization
//! - `execute_operation` – Move, Copy, Delete, Rename
//! - `undo_operation`     – Revert a completed operation
//! - `get_folders`        – Retrieve all configured folders
//! - `manage_folders`     – CRUD operations on folders
//! - `settings`           – Load and save user settings
//! - `plugin_manager`     – List, enable, disable plugins
//! - `search_files`       – Search files by query
//!
//! All use cases are re-exported for convenient access by the application facade.

// each sub-module is now declared with a brief comment for clarity

mod execute_operation; // ExecuteOperationUseCase – Move, Copy, Delete, Rename
mod get_folders; // GetFoldersUseCase – list all configured folders
mod get_operation_history; // GetOperationHistoryUseCase – retrieve operation history
mod manage_folders;
mod plugin_manager; // PluginManagerUseCase – plugin lifecycle management
mod search_files; // SearchFilesUseCase – search files by query
mod settings; // LoadSettingsUseCase, SaveSettingsUseCase – user settings
mod undo_operation; // UndoOperationUseCase – revert completed operations

// Re-export the concrete use case types so they can be used directly
// from `crate::use_cases::*`.
pub use execute_operation::ExecuteOperationUseCase;
pub use get_folders::GetFoldersUseCase;
pub use get_operation_history::GetOperationHistoryUseCase;
pub use manage_folders::ManageFoldersUseCase;
pub use plugin_manager::{PluginConfigRepository, PluginLoader, PluginManagerUseCase};
pub use search_files::{SearchFiles, SearchFilesUseCase};
pub use settings::{LoadSettingsUseCase, SaveSettingsUseCase};
pub use undo_operation::UndoOperationUseCase;
