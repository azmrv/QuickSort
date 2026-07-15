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
//!
//! All use cases are re-exported for convenient access by the application facade.

// OLD: simple `mod` declarations without documentation
// NEW: each sub-module is now declared with a brief comment for clarity

mod execute_operation;   // ExecuteOperationUseCase – Move, Copy, Delete, Rename
mod undo_operation;      // UndoOperationUseCase – revert completed operations
mod get_folders;         // GetFoldersUseCase – list all configured folders
mod manage_folders;      // ManageFoldersUseCase – add, remove, rename folders

// Re-export the concrete use case types so they can be used directly
// from `crate::use_cases::*`.
pub use execute_operation::ExecuteOperationUseCase;
pub use undo_operation::UndoOperationUseCase;
pub use get_folders::GetFoldersUseCase;
pub use manage_folders::ManageFoldersUseCase;