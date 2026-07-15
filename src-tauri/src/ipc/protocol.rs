//! IPC protocol definitions – **DEPRECATED**.
//!
//! The canonical versions of these types now live in
//! `quicksort-ipc-contract`.  This file is kept temporarily so that the
//! existing pipe server can compile while it is being migrated.
//!
//! # Migration path
//! 1. Update `pipe_server::server.rs` to import types from
//!    `quicksort_ipc_contract` instead of from this module.
//! 2. Delete this file.
//!
//! # Why this was temporarily duplicated
//! During the transition from a monolithic Tauri crate to a multi-crate
//! workspace, the IPC protocol types were needed in both the DLL and the
//! Tauri adapter.  Moving them into a dedicated crate (`quicksort-ipc-contract`)
//! eliminates the duplication.  Until the server module is fully updated,
//! this file serves as a bridge.

use serde::{Deserialize, Serialize};

// Re-export from the canonical crate so that any code importing from here
// continues to work.  New code should import directly from `quicksort_ipc_contract`.
pub use quicksort_ipc_contract::{
    PROTOCOL_VERSION, MAGIC,
    CommandMessage, ExecuteOperationData, OperationType, OverwritePolicy,
    ResponseMessage, ResponseStatus,
};

// The old types below are no longer used, but are kept as comments for
// reference during the migration.

// OLD: pub struct PipeCommand { ... }
// OLD: pub enum PipeAction { ... }
// OLD: pub struct OperationCommand { ... }
// (all replaced by the types in quicksort-ipc-contract)