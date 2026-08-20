//! IPC (Inter-Process Communication) module for the Tauri adapter.
//!
//! This module contains the Named Pipe server that receives commands from
//! the shell extension DLL.  It uses the framing protocol defined in
//! `quicksort-ipc-contract` and forwards decoded commands to the
//! Application Facade.

pub mod framing;
pub mod protocol;
pub mod server;
