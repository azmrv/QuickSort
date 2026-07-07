//! IPC error types.

use windows::core::Error as WindowsError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PipeError {
    #[error("Pipe not available")]
    Unavailable,
    #[error("Pipe busy")]
    Busy,
    #[error("Windows error: {0}")]
    Windows(#[from] WindowsError),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Incomplete write: {0}/{1} bytes")]
    IncompleteWrite(u32, usize),
    #[error("Incomplete read: expected {expected}, got {actual}")]
    IncompleteRead { expected: u32, actual: u32 },
    #[error("Protocol version mismatch: expected {expected}, got {actual}")]
    VersionMismatch { expected: u16, actual: u16 },
    #[error("Invalid magic bytes")]
    InvalidMagic,
    #[error("Message too large: {size} (max: {max})")]
    MessageTooLarge { size: u32, max: u32 },
    #[error("Operation timed out")]
    Timeout,
    #[error("Operation failed: {0}")]
    OperationFailed(String),
}