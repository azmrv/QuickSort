//! Data Transfer Objects (DTOs) for the Application Layer.
//!
//! DTOs are plain data structures that cross the boundary between the
//! Application Layer and its adapters (Tauri commands, IPC handlers, CLI).
//! They are **not** domain entities and contain no business logic.
//!
//! # Design Decisions
//! - DTOs use domain types (`OperationId`, `WindowsPath`, `FolderId`) where
//!   appropriate to avoid anemic wrappers and unnecessary mapping.
//! - `OperationCommand` and `OperationResult` are the primary contracts for
//!   file operations (Move, Copy, Delete, Rename, Undo).
//! - `PipeCommand` / `PipeAction` are IPC-specific envelopes used by the
//!   Named Pipe server to decode messages from the shell extension DLL.
//!
//! # Module Organization
//! | Module | Purpose |
//! |--------|---------|
//! | `operation_command` | Input DTO for `ExecuteOperation` |
//! | `operation_result` | Output DTO returned by all file operations |
//! | `pipe_command` | IPC envelope for Named Pipe communication |
//!
//! # Future Extensions
//! - `UndoOperationResult` – may diverge from `OperationResult` if undo
//!   requires additional metadata.
//! - `FolderConfigDTO` – for bulk folder operations (e.g., import/export).

mod operation_command;
mod operation_result;
mod pipe_command;

pub use operation_command::{OperationCommand, OverwritePolicy};
pub use operation_result::OperationResult;
pub use pipe_command::{PipeCommand, PipeAction};