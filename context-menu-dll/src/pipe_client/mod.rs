//! IPC client for communicating with the Tauri app.

mod error;
pub mod transport;
pub mod client;
pub mod win32_transport;

pub use error::PipeError;
pub use transport::named_pipe::NamedPipeTransport;
pub use transport::pipe_trait::PipeTransport;
pub use client::{PipeClient, move_to_folder, ping};

