//! Transport abstraction for IPC.

use crate::pipe_client::error::PipeError;

pub trait PipeTransport: Send + Sync {
    /// Opens the connection.
    fn connect(&mut self) -> Result<(), PipeError>;

    /// Writes raw bytes (framing is handled by upper layers).
    fn write(&mut self, data: &[u8]) -> Result<(), PipeError>;
}