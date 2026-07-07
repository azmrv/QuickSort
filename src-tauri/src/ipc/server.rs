//! Named Pipe server for receiving commands from the shell extension DLL.

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::sync::Arc;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{
    CloseHandle, GetLastError, ERROR_PIPE_BUSY, HANDLE, INVALID_HANDLE_VALUE,
};
use windows::Win32::Storage::FileSystem::{
    FlushFileBuffers, FILE_FLAG_FIRST_PIPE_INSTANCE, FILE_SHARE_READ, FILE_SHARE_WRITE,
};
use windows::Win32::System::Pipes::{
    ConnectNamedPipe, CreateNamedPipeW, PIPE_ACCESS_DUPLEX, PIPE_TYPE_BYTE,
    PIPE_READMODE_BYTE, PIPE_WAIT, PIPE_UNLIMITED_INSTANCES,
};

use crate::pipe_server::protocol::*;
use crate::pipe_server::framing::{read_frame, write_frame};

const PIPE_NAME: &str = r"\\.\pipe\quicksort_cmd";

/// RAII wrapper for pipe HANDLE.
struct PipeHandle(HANDLE);

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

/// Starts the pipe server loop. This function blocks, so run it in a separate thread.
pub fn start_pipe_server() {
    log::info!("Pipe server starting...");

    let pipe_name: Vec<u16> = OsStr::new(PIPE_NAME)
        .encode_wide()
        .chain(Some(0))
        .collect();

    loop {
        // Create the named pipe
        let handle = unsafe {
            CreateNamedPipeW(
                PCWSTR::from_raw(pipe_name.as_ptr()),
                PIPE_ACCESS_DUPLEX | FILE_FLAG_FIRST_PIPE_INSTANCE,
                PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT,
                PIPE_UNLIMITED_INSTANCES,
                512,   // out buffer size
                512,   // in buffer size
                0,     // default timeout
                None,  // security attributes
            )
        };

        if handle == INVALID_HANDLE_VALUE {
            let err = unsafe { GetLastError() };
            log::error!("CreateNamedPipeW failed: {:?}", err);
            std::thread::sleep(std::time::Duration::from_secs(1));
            continue;
        }

        let pipe = PipeHandle(handle);

        // Wait for client to connect
        unsafe {
            ConnectNamedPipe(pipe.raw(), None);
        }

        log::info!("Client connected to pipe");

        // Process requests
        loop {
            match read_frame(pipe.raw()) {
                Ok(data) => {
                    match serde_json::from_slice::<PipeCommand>(&data) {
                        Ok(cmd) => {
                            if cmd.version != PROTOCOL_VERSION {
                                log::error!("Protocol version mismatch");
                                let resp = ResponseMessage {
                                    success: false,
                                    error: Some("Version mismatch".to_string()),
                                };
                                let _ = write_frame(pipe.raw(), &serde_json::to_vec(&resp).unwrap());
                                continue;
                            }
                            match cmd.action {
                                PipeAction::ExecuteOperation { command } => {
                                    // TODO: call the actual UseCase (ExecuteOperationUseCase)
                                    log::info!("Received ExecuteOperation: {:?}", command);
                                    let success = true; // placeholder
                                    let resp = ResponseMessage {
                                        success,
                                        error: None,
                                    };
                                    let response_bytes = serde_json::to_vec(&resp).unwrap();
                                    if let Err(e) = write_frame(pipe.raw(), &response_bytes) {
                                        log::error!("Failed to send response: {}", e);
                                        break;
                                    }
                                    // Flush to ensure client receives the response
                                    unsafe { FlushFileBuffers(pipe.raw()).ok(); }
                                }
                            }
                        }
                        Err(e) => {
                            log::error!("Failed to deserialize command: {}", e);
                            let resp = ResponseMessage {
                                success: false,
                                error: Some(format!("Deserialization error: {}", e)),
                            };
                            let _ = write_frame(pipe.raw(), &serde_json::to_vec(&resp).unwrap());
                        }
                    }
                }
                Err(e) => {
                    log::error!("Read error: {}", e);
                    break;
                }
            }
        }
    }
}