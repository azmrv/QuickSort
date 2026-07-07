//! Win32 Named Pipe transport.

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::time::Duration;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{
    CloseHandle, GetLastError, ERROR_PIPE_BUSY, HANDLE,
    GENERIC_READ, GENERIC_WRITE,
};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, ReadFile, WriteFile, FILE_SHARE_READ, FILE_SHARE_WRITE,
    OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL,
};
use windows::Win32::System::Pipes::WaitNamedPipeW;
use crate::pipe_client::error::PipeError;
use crate::pipe_client::protocol::*;  // <-- добавлен импорт
use super::PipeTransport;

const PIPE_NAME: &str = r"\\.\pipe\quicksort_cmd";
const CONNECT_TIMEOUT_MS: u32 = 100;
const READ_TIMEOUT_MS: u32 = 1000;
const WRITE_TIMEOUT_MS: u32 = 1000;
const MAX_RETRIES: u32 = 3;
const RETRY_INTERVAL_MS: u64 = 20;

pub struct PipeHandle(HANDLE);

impl PipeHandle {
    pub fn new(handle: HANDLE) -> Self {
        Self(handle)
    }

    pub fn as_handle(&self) -> HANDLE {
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

pub struct NamedPipeTransport {
    handle: Option<PipeHandle>,
    retries: u32,
}

impl NamedPipeTransport {
    pub fn new() -> Self {
        Self {
            handle: None,
            retries: MAX_RETRIES,
        }
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

            unsafe {
                let wait_result = WaitNamedPipeW(pipe_name, CONNECT_TIMEOUT_MS);
                if !wait_result.as_bool() {
                    let err = GetLastError();
                    if err == ERROR_PIPE_BUSY {
                        if attempts >= self.retries {
                            return Err(PipeError::Busy);
                        }
                        std::thread::sleep(Duration::from_millis(RETRY_INTERVAL_MS));
                        continue;
                    } else {
                        return Err(PipeError::Unavailable);
                    }
                }
            }

            let handle = unsafe {
                CreateFileW(
                    pipe_name,
                    GENERIC_READ.0 | GENERIC_WRITE.0,
                    FILE_SHARE_READ | FILE_SHARE_WRITE,
                    None,
                    OPEN_EXISTING,
                    FILE_ATTRIBUTE_NORMAL,
                    None,
                )
            }?;

            self.handle = Some(PipeHandle::new(handle));
            return Ok(());
        }
    }

    fn send(&mut self, data: &[u8]) -> Result<(), PipeError> {
        let handle = self.handle.as_ref()
            .ok_or(PipeError::Unavailable)?;

        let mut bytes_written = 0u32;
        unsafe {
            WriteFile(
                handle.as_handle(),
                Some(data),
                Some(&mut bytes_written),
                None,
            )?;
        }

        if bytes_written as usize != data.len() {
            return Err(PipeError::IncompleteWrite(bytes_written, data.len()));
        }

        Ok(())
    }

    fn receive(&mut self) -> Result<Vec<u8>, PipeError> {
        let handle = self.handle.as_ref()
            .ok_or(PipeError::Unavailable)?;

        // Read header first (12 bytes)
        let mut header_buf = [0u8; 12];
        let mut bytes_read = 0u32;

        unsafe {
            ReadFile(
                handle.as_handle(),
                Some(&mut header_buf),
                Some(&mut bytes_read),
                None,
            )?;
        }

        if bytes_read as usize != header_buf.len() {
            return Err(PipeError::IncompleteRead {
                expected: header_buf.len() as u32,
                actual: bytes_read,
            });
        }

        let header = MessageHeader::decode(&header_buf)?;
        let payload_len = header.length as usize;

        // Read payload
        let mut payload = vec![0u8; payload_len];
        let mut bytes_read = 0u32;

        unsafe {
            ReadFile(
                handle.as_handle(),
                Some(&mut payload),
                Some(&mut bytes_read),
                None,
            )?;
        }

        if bytes_read as usize != payload_len {
            return Err(PipeError::IncompleteRead {
                expected: payload_len as u32,
                actual: bytes_read,
            });
        }

        let mut result = Vec::with_capacity(header_buf.len() + payload_len);
        result.extend_from_slice(&header_buf);
        result.extend_from_slice(&payload);

        Ok(result)
    }

    fn disconnect(&mut self) -> Result<(), PipeError> {
        self.handle = None;
        Ok(())
    }
}