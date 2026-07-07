//! Win32 implementation of PipeTransport using named pipes (synchronous).

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::time::Duration;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{
    CloseHandle, GetLastError, ERROR_PIPE_BUSY, HANDLE,
    GENERIC_READ, GENERIC_WRITE,
};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, WriteFile, FILE_SHARE_READ, FILE_SHARE_WRITE,
    OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL,
};
use windows::Win32::System::Pipes::WaitNamedPipeW;
use crate::pipe_client::error::PipeError;
use super::transport::PipeTransport;

const PIPE_NAME: &str = r"\\.\pipe\quicksort_cmd";
const MAX_RETRIES: u32 = 5;
const RETRY_INTERVAL_MS: u64 = 20;

/// RAII wrapper for HANDLE.
struct PipeHandle(HANDLE);

impl PipeHandle {
    fn new(handle: HANDLE) -> Self {
        Self(handle)
    }
    fn as_handle(&self) -> HANDLE {
        self.0
    }
}

impl Drop for PipeHandle {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

/// Named pipe transport.
pub struct NamedPipeTransport {
    handle: Option<PipeHandle>,
}

impl NamedPipeTransport {
    pub fn new() -> Self {
        Self { handle: None }
    }
}

impl PipeTransport for NamedPipeTransport {
    fn connect(&mut self) -> Result<(), PipeError> {
        let pipe_name_wide: Vec<u16> = OsStr::new(PIPE_NAME)
            .encode_wide()
            .chain(Some(0))
            .collect();

        let pipe_name = PCWSTR::from_raw(pipe_name_wide.as_ptr());

        let mut attempts = 0;
        loop {
            attempts += 1;

            // Wait for pipe with short timeout
            unsafe {
                let wait_result = WaitNamedPipeW(pipe_name, 20);
                if !wait_result.as_bool() {
                    let err = GetLastError();
                    if err == ERROR_PIPE_BUSY {
                        if attempts >= MAX_RETRIES {
                            return Err(PipeError::Busy);
                        }
                        std::thread::sleep(Duration::from_millis(RETRY_INTERVAL_MS));
                        continue;
                    } else {
                        return Err(PipeError::Unavailable);
                    }
                }
            }

            // Open pipe (synchronous, no overlapped)
            // GENERIC_READ/GENERIC_WRITE are structs; use .0 to get u32 mask.
            let desired_access = GENERIC_READ.0 | GENERIC_WRITE.0;

            match unsafe {
                CreateFileW(
                    pipe_name,
                    desired_access,
                    FILE_SHARE_READ | FILE_SHARE_WRITE,
                    None,
                    OPEN_EXISTING,
                    FILE_ATTRIBUTE_NORMAL,
                    None,
                )
            } {
                Ok(handle) => {
                    // In windows-rs 0.62, CreateFileW returns Result<HANDLE>.
                    // If it succeeded, the handle is valid (no INVALID_HANDLE_VALUE check needed).
                    self.handle = Some(PipeHandle::new(handle));
                    return Ok(());
                }
                Err(e) => {
                    if attempts >= MAX_RETRIES {
                        return Err(PipeError::Windows(e));
                    }
                    std::thread::sleep(Duration::from_millis(RETRY_INTERVAL_MS));
                }
            }
        }
    }

    fn write(&mut self, data: &[u8]) -> Result<(), PipeError> {
        let handle = self.handle.as_ref()
            .ok_or(PipeError::Unavailable)?;

        let len = data.len() as u32;
        let len_bytes = len.to_le_bytes();

        // Write length prefix
        let mut written = 0u32;
        unsafe {
            WriteFile(
                handle.as_handle(),
                Some(&len_bytes),
                Some(&mut written),
                None,
            )?;
        }
        if written != 4 {
            return Err(PipeError::IncompleteWrite(written, 4));
        }

        // Write payload
        let mut written = 0u32;
        unsafe {
            WriteFile(
                handle.as_handle(),
                Some(data),
                Some(&mut written),
                None,
            )?;
        }
        if written as usize != data.len() {
            return Err(PipeError::IncompleteWrite(written, data.len()));
        }

        Ok(())
    }
}