//! Framing utilities for length-prefixed messages over a Named Pipe.
//!
//! The wire format is extremely simple:
//!   [u32 LE – payload length][payload bytes]
//!
//! Both sides (DLL client and Tauri server) MUST use this exact format.
//! The functions in this module are the **only** code that directly calls
//! `ReadFile` / `WriteFile` with the pipe handle; all other code works
//! with plain `Vec<u8>`.

use windows::Win32::Foundation::HANDLE;
use windows::Win32::Storage::FileSystem::{ReadFile, WriteFile};

/// Reads a single complete framed message from `handle`.
///
/// # Protocol
/// 1. Read exactly 4 bytes – the payload length encoded as little-endian `u32`.
/// 2. Read exactly `payload_len` bytes – the actual message body.
///
/// # Errors
/// Returns `Err` if:
/// - The pipe is closed before the full length prefix or body is received.
/// - `ReadFile` itself fails (e.g., broken pipe, permissions).
pub fn read_frame(handle: HANDLE) -> Result<Vec<u8>, String> {
    // ---- Step 1: Read the length prefix (4 bytes, little-endian) ----
    let mut len_buf = [0u8; 4];
    let mut bytes_read = 0u32;
    unsafe {
        ReadFile(
            handle,
            Some(&mut len_buf),
            Some(&mut bytes_read),
            None,                 // synchronous read (no OVERLAPPED)
        )
        .map_err(|e| format!("ReadFile (length prefix) failed: {e:?}"))?;
    }
    if bytes_read != 4 {
        return Err(format!(
            "Incomplete length prefix read: {bytes_read}/4 bytes"
        ));
    }
    let payload_len = u32::from_le_bytes(len_buf) as usize;

    // ---- Step 2: Read exactly payload_len bytes ----
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
            .map_err(|e| format!("ReadFile (payload) failed: {e:?}"))?;
        }
        // A zero-length read with more bytes expected means the pipe was
        // closed prematurely by the remote side.
        if chunk_read == 0 {
            return Err(format!(
                "Pipe closed after {total_read}/{payload_len} payload bytes"
            ));
        }
        total_read += chunk_read as usize;
    }
    Ok(payload)
}

/// Writes a complete framed message to `handle`.
///
/// # Protocol
/// 1. Write the payload length as a 4-byte little-endian `u32`.
/// 2. Write the actual payload bytes.
///
/// # Errors
/// Returns `Err` if `WriteFile` fails or the number of bytes written
/// does not match the expected count.
pub fn write_frame(handle: HANDLE, data: &[u8]) -> Result<(), String> {
    let len = data.len() as u32;

    // ---- Step 1: Write the length prefix ----
    let len_bytes = len.to_le_bytes();
    let mut written = 0u32;
    unsafe {
        WriteFile(
            handle,
            Some(&len_bytes),
            Some(&mut written),
            None,                 // synchronous write
        )
        .map_err(|e| format!("WriteFile (length prefix) failed: {e:?}"))?;
    }
    if written != 4 {
        return Err(format!(
            "Incomplete length prefix write: {written}/4 bytes"
        ));
    }

    // ---- Step 2: Write the payload ----
    let mut written = 0u32;
    unsafe {
        WriteFile(
            handle,
            Some(data),
            Some(&mut written),
            None,
        )
        .map_err(|e| format!("WriteFile (payload) failed: {e:?}"))?;
    }
    if written as usize != data.len() {
        return Err(format!(
            "Incomplete payload write: {written}/{} bytes",
            data.len()
        ));
    }
    Ok(())
}