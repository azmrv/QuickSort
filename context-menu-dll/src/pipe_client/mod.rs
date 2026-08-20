//! IPC client for communicating with the Tauri app.

pub mod client;
mod error;
pub mod transport;

pub use client::move_to_folder;
