//! Framing utilities for length-prefixed messages.

use windows::core::Result as WinResult;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Storage::FileSystem::{ReadFile, WriteFile};

/// Reads a single framed message from the pipe handle.
/// Returns the payload bytes on success.
pub fn read_frame(handle: HANDLE) -> Result<Vec<u8>, String> {
    // 1. Read length prefix (4 bytes, little-endian)
    let mut len_buf = [0u8; 4];
    let mut bytes_read = 0u32;
    unsafe {
        ReadFile(
            handle,
            Some(&mut len_buf),
            Some(&mut bytes_read),
            None,
        )
            .map_err(|e| format!("ReadFile failed: {:?}", e))?;
    }
    if bytes_read != 4 {
        return Err(format!("Incomplete length read: {}/4", bytes_read));
    }
    let payload_len = u32::from_le_bytes(len_buf) as usize;

    // 2. Read exactly payload_len bytes
    let mut payload = vec![0u8; payload_len];
    let mut total_read = 0usize;
    while total_read < payload_len {
        let mut chunk_read = 0u32;
        unsafe {
            ReadFile(
                handle,
                Some(&mut payload[total_read..]),
                Some(&mut chunk_read),
                None,
            )
                .map_err(|e| format!("ReadFile failed: {:?}", e))?;
        }
        if chunk_read == 0 {
            return Err("Pipe closed before complete message".to_string());
        }
        total_read += chunk_read as usize;
    }
    Ok(payload)
}

/// Writes a framed message (length prefix + data) to the pipe handle.
pub fn write_frame(handle: HANDLE, data: &[u8]) -> Result<(), String> {
    let len = data.len() as u32;
    let len_bytes = len.to_le_bytes();

    // Write length prefix
    let mut written = 0u32;
    unsafe {
        WriteFile(
            handle,
            Some(&len_bytes),
            Some(&mut written),
            None,
        )
            .map_err(|e| format!("WriteFile failed: {:?}", e))?;
    }
    if written != 4 {
        return Err(format!("Incomplete length write: {}/4", written));
    }

    // Write payload
    let mut written = 0u32;
    unsafe {
        WriteFile(
            handle,
            Some(data),
            Some(&mut written),
            None,
        )
            .map_err(|e| format!("WriteFile failed: {:?}", e))?;
    }
    if written as usize != data.len() {
        return Err(format!("Incomplete data write: {}/{}", written, data.len()));
    }
    Ok(())
}