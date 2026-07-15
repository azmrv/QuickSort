//! Unified IPC protocol for QuickSort.
//!
//! This crate defines the **language-independent contract** between the
//! shell extension DLL (client) and the main Tauri application (server).
//! Both components depend on this crate so that a change to the protocol
//! requires a single update, not duplicated edits.
//!
//! # Protocol Overview
//! | Direction | Message Type | Purpose |
//! |-----------|-------------|---------|
//! | Client → Server | `CommandMessage` | Request an operation (Move, Copy, Delete, Rename) or Ping |
//! | Server → Client | `ResponseMessage` | Confirms success or reports an error |
//!
//! # Framing
//! Messages are transmitted over a Named Pipe with a simple length-prefixed
//! framing scheme: `[u32 LE length][JSON body]`.  The `MAGIC` constant is
//! reserved for future framing validation (e.g., for detecting corrupt
//! streams).
//!
//! # Versioning
//! `PROTOCOL_VERSION` is incremented when backward-incompatible changes are
//! made.  The server and client MUST agree on the same version before any
//! data is exchanged.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Protocol constants
// ---------------------------------------------------------------------------

/// Current protocol version.
///
/// Increment this whenever the wire format changes in a way that is **not**
/// backward-compatible.  The server and client MUST check the version before
/// processing any message.
pub const PROTOCOL_VERSION: u16 = 1;

/// Magic number reserved for future use (e.g., framing validation).
/// Currently unused; stored here as a documentation artefact.
pub const MAGIC: u32 = 0x51535452; // "QSTR" in little-endian ASCII

// ---------------------------------------------------------------------------
// Client → Server messages
// ---------------------------------------------------------------------------

/// A command sent from the client (DLL) to the server (Tauri).
///
/// Serialized as JSON with a "type" discriminator, so the wire format looks
/// like `{"type": "ExecuteOperation", "data": {...}}` or `{"type": "Ping"}`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum CommandMessage {
    /// Request the execution of a file operation (Move, Copy, Delete, Rename).
    ExecuteOperation(ExecuteOperationData),

    /// A simple keep-alive or health-check request.
    /// The server should respond with `ResponseMessage { status: Ok }`.
    Ping,
}

/// Payload for the `ExecuteOperation` command.
///
/// All fields mirror those of `quicksort_application::OperationCommand`,
/// but use plain `String` for paths and IDs to avoid dragging domain types
/// into the IPC contract.  The server is responsible for validating and
/// converting these into domain value objects.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteOperationData {
    /// The type of operation to perform.
    pub operation_type: OperationType,

    /// Absolute paths of the files to operate on.
    ///
    /// Must contain at least one entry.  Paths use the Windows backslash
    /// format (e.g., `C:\Users\...`).
    pub source_paths: Vec<String>,

    /// UUID of the target folder, for Move/Copy operations.
    ///
    /// Must be `Some` for Move and Copy, `None` for Delete and Rename.
    pub target_folder_id: Option<String>,

    /// Conflict resolution strategy when a destination file already exists.
    pub overwrite_policy: OverwritePolicy,
}

// ---------------------------------------------------------------------------
// Enumerations shared between client and server
// ---------------------------------------------------------------------------

/// The kind of file operation to perform.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationType {
    /// Move the source file(s) to the target folder.
    Move,
    /// Copy the source file(s) to the target folder, leaving the originals.
    Copy,
    /// Permanently delete the source file(s).
    Delete,
    /// Change the name (and optionally the path) of the source file(s).
    Rename,
}

/// Conflict resolution strategy when a file already exists at the destination.
///
/// This enum mirrors `quicksort_application::OverwritePolicy`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OverwritePolicy {
    /// Abort the operation and report a conflict error.
    Skip,
    /// Silently replace the existing file.
    Overwrite,
    /// Append a numeric suffix to create a unique file name
    /// (e.g., "file (1).txt").
    AutoRename,
    /// Prompt the user for a decision.  In non-interactive contexts
    /// (e.g., when the command comes from the shell extension DLL),
    /// this policy falls back to `AutoRename`.
    Ask,
}

// ---------------------------------------------------------------------------
// Server → Client messages
// ---------------------------------------------------------------------------

/// A response sent from the server (Tauri) to the client (DLL).
///
/// Every `CommandMessage` (except `Ping`, which is optional to respond to)
/// MUST be acknowledged with a `ResponseMessage`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMessage {
    /// Overall status of the request.
    pub status: ResponseStatus,

    /// Human-readable description of the outcome.
    /// On success this may be empty; on error it contains a user-friendly message.
    pub message: String,

    /// The UUID of the operation, if one was created.
    /// This can be used for future undo requests.
    pub operation_id: Option<String>,

    /// Additional structured data, if applicable.
    /// For example, the server may return detailed per-file results here.
    pub data: Option<serde_json::Value>,
}

/// Status codes returned by the server.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ResponseStatus {
    /// The operation was completed successfully.
    Ok,
    /// An error occurred during processing.
    Error,
    /// The operation was accepted but has not yet completed.
    /// Reserved for future use with asynchronous processing.
    Pending,
}