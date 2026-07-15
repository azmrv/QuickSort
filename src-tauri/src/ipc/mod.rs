//! IPC (Inter-Process Communication) module for the Tauri adapter.
//!
//! This module contains the Named Pipe server that receives commands from
//! the shell extension DLL.  It uses the framing protocol defined in
//! `quicksort-ipc-contract` and forwards decoded commands to the
//! Application Facade.

pub mod protocol;   // Protocol types (temporary; will move to quicksort-ipc-contract)
pub mod framing;    // Low-level length-prefixed frame I/O
pub mod server;     // Pipe server lifecycle and request handling

pub use server::start_pipe_server;