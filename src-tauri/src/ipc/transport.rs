//! Platform-agnostic IPC transport trait.
//!
//! On Windows, the transport uses Named Pipes (`\\.\pipe\quicksort_cmd`).
//! On Linux/macOS, the transport uses Unix Domain Sockets.

use std::io;

/// A platform-agnostic IPC transport that can accept connections.
pub trait IpcTransport: Send + Sync {
    /// The type representing a connected client stream.
    type Stream: IpcStream;

    /// Start listening for connections.
    fn start(&self) -> io::Result<()>;

    /// Accept the next incoming connection.
    fn accept(&self) -> io::Result<Self::Stream>;

    /// Clean up the transport (remove socket files, etc.)
    #[allow(dead_code)] // Used on non-Windows platforms during server shutdown
    fn cleanup(&self);

    /// Get a human-readable name for logging.
    fn name(&self) -> &str;
}

/// A platform-agnostic IPC stream for reading/writing framed messages.
pub trait IpcStream: Send + 'static {
    /// Read a complete framed message.
    fn read_frame(&mut self) -> io::Result<Vec<u8>>;

    /// Write a complete framed message.
    fn write_frame(&mut self, data: &[u8]) -> io::Result<()>;
}
