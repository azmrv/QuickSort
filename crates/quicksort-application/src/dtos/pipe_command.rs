//! DTO for IPC communication between DLL (shell extension) and Tauri (GUI).
//!
//! These types mirror the IPC contract defined in `quicksort-ipc-contract`
//! but are kept inside the Application layer to avoid a direct dependency
//! on the IPC crate.  Adapters (the pipe server) map between the two.
//!
//! # Design Decision
//! The Application layer should not know *how* commands arrive (Named Pipe,
//! COM, HTTP).  Therefore the IPC-specific envelope (`PipeCommand`,
//! `PipeAction`) is defined here as a simple DTO, while the actual
//! `OperationCommand` is the canonical application DTO used by all Use Cases.

use serde::{Deserialize, Serialize};
use crate::dtos::OperationCommand;

/// Top-level envelope for every message sent over the named pipe.
///
/// The `version` field allows the protocol to evolve without breaking
/// existing clients.  Currently only version 1 is supported.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipeCommand {
    /// Protocol version (currently 1).
    pub version: u32,

    /// The action that the server should perform.
    pub action: PipeAction,
}

/// Enumeration of all possible actions that can be requested via IPC.
///
/// Tagged with `#[serde(tag = "type")]` so the JSON looks like:
/// `{"type": "ExecuteOperation", "command": {...}}`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PipeAction {
    /// Execute a file operation (Move, Copy, Delete, Rename).
    /// The enclosed `OperationCommand` is the same DTO used by
    /// `ExecuteOperationUseCase`.
    ExecuteOperation {
        command: OperationCommand,
    },

    // Future extensions:
    // GetFolders,
    // Ping,
    // UndoOperation { operation_id: String },
}