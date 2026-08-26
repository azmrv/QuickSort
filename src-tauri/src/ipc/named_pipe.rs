//! Windows Named Pipe transport implementation.
//!
//! Uses `\\.\pipe\quicksort_cmd` for IPC with the shell extension DLL.

use std::ffi::OsStr;
use std::io;
use std::os::windows::ffi::OsStrExt;

use windows::core::PCWSTR;
use windows::Win32::Foundation::{CloseHandle, GetLastError, HANDLE, INVALID_HANDLE_VALUE};
use windows::Win32::Storage::FileSystem::{
    FlushFileBuffers, FILE_FLAG_FIRST_PIPE_INSTANCE, PIPE_ACCESS_DUPLEX,
};
use windows::Win32::System::Pipes::{
    ConnectNamedPipe, CreateNamedPipeW, PIPE_READMODE_BYTE, PIPE_TYPE_BYTE,
    PIPE_UNLIMITED_INSTANCES, PIPE_WAIT,
};

use super::transport::{IpcStream, IpcTransport};

const PIPE_NAME: &str = r"\\.\pipe\quicksort_cmd";

// ---------------------------------------------------------------------------
// RAII wrapper for HANDLE
// ---------------------------------------------------------------------------

struct PipeHandle(HANDLE);

// SAFETY: Windows HANDLEs are kernel objects that are safe to send between
// threads. The windows crate does not implement Send for HANDLE because raw
// pointers are not inherently safe, but kernel handles are valid across threads.
unsafe impl Send for PipeHandle {}

impl PipeHandle {
    fn raw(&self) -> HANDLE {
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

// ---------------------------------------------------------------------------
// Named Pipe Stream
// ---------------------------------------------------------------------------

/// A connected Named Pipe client stream.
pub struct NamedPipeStream {
    handle: PipeHandle,
}

impl IpcStream for NamedPipeStream {
    fn read_frame(&mut self) -> io::Result<Vec<u8>> {
        super::framing::read_frame(self.handle.raw()).map_err(io::Error::other)
    }

    fn write_frame(&mut self, data: &[u8]) -> io::Result<()> {
        super::framing::write_frame(self.handle.raw(), data).map_err(io::Error::other)?;
        unsafe {
            FlushFileBuffers(self.handle.raw()).ok();
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Named Pipe Transport
// ---------------------------------------------------------------------------

/// Windows Named Pipe IPC transport.
pub struct NamedPipeTransport {
    pipe_name_wide: Vec<u16>,
}

impl NamedPipeTransport {
    pub fn new() -> Self {
        let pipe_name_wide: Vec<u16> = OsStr::new(PIPE_NAME).encode_wide().chain(Some(0)).collect();
        Self { pipe_name_wide }
    }
}

impl IpcTransport for NamedPipeTransport {
    type Stream = NamedPipeStream;

    fn start(&self) -> io::Result<()> {
        tracing::info!("Named Pipe transport starting on {}", PIPE_NAME);
        Ok(())
    }

    fn accept(&self) -> io::Result<NamedPipeStream> {
        let handle = unsafe {
            CreateNamedPipeW(
                PCWSTR::from_raw(self.pipe_name_wide.as_ptr()),
                PIPE_ACCESS_DUPLEX | FILE_FLAG_FIRST_PIPE_INSTANCE,
                PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT,
                PIPE_UNLIMITED_INSTANCES,
                4096,
                4096,
                0,
                None,
            )
        };

        if handle == INVALID_HANDLE_VALUE {
            let err = unsafe { GetLastError() };
            return Err(io::Error::other(format!(
                "CreateNamedPipeW failed: {:?}",
                err
            )));
        }

        let pipe = PipeHandle(handle);

        unsafe {
            let _ = ConnectNamedPipe(pipe.raw(), None);
        }
        tracing::info!("Client connected to pipe");

        Ok(NamedPipeStream { handle: pipe })
    }

    #[allow(dead_code)] // Called during server shutdown on other platforms
    fn cleanup(&self) {
        // Named Pipes are cleaned up by the OS when all handles are closed.
    }

    fn name(&self) -> &str {
        "NamedPipe"
    }
}
