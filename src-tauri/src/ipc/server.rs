//! Named Pipe server that receives commands from the shell extension DLL.
//!
//! This module runs in a dedicated background thread and listens for
//! incoming connections on `\\.\pipe\quicksort_cmd`.  Every command
//! received is deserialized, validated, and forwarded to the Application
//! Facade for execution.  A response is sent back to the client (DLL).
//!
//! # Design Decisions
//! - The server uses a single-threaded, synchronous I/O model because
//!   command volume is low (a few per minute) and simplicity is preferred.
//! - The server is **not** responsible for retry or error recovery – that
//!   is handled by the client (DLL) via its own retry policy.
//! - Once the Application Facade is fully integrated, the TODO marker
//!   below will be replaced with an actual `facade.execute_operation(...)`
//!   call.

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{
    CloseHandle, GetLastError, HANDLE, INVALID_HANDLE_VALUE,
};
use windows::Win32::Storage::FileSystem::{
    FlushFileBuffers, FILE_FLAG_FIRST_PIPE_INSTANCE, FILE_SHARE_READ,
    FILE_SHARE_WRITE,
};
use windows::Win32::System::Pipes::{
    ConnectNamedPipe, CreateNamedPipeW, PIPE_ACCESS_DUPLEX, PIPE_TYPE_BYTE,
    PIPE_READMODE_BYTE, PIPE_WAIT, PIPE_UNLIMITED_INSTANCES,
};

// Import the canonical IPC contract – no more duplicated DTOs.
use quicksort_ipc_contract::{
    CommandMessage, ResponseMessage, ResponseStatus, PROTOCOL_VERSION,
};

use super::framing::{read_frame, write_frame};

/// The well-known name of the pipe.  Both the DLL and this server MUST agree
/// on this value.
const PIPE_NAME: &str = r"\\.\pipe\quicksort_cmd";

// ---------------------------------------------------------------------------
// RAII wrapper for HANDLE
// ---------------------------------------------------------------------------

/// Owns a pipe `HANDLE` and closes it on drop.
///
/// Using RAII guarantees that the handle is released even if an error occurs
/// or the thread panics.
struct PipeHandle(HANDLE);

impl PipeHandle {
    /// Returns the underlying `HANDLE` for read/write operations.
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
// Public entry point
// ---------------------------------------------------------------------------

/// Starts the pipe server loop.
///
/// # Blocking
/// This function never returns under normal operation.  It must be spawned
/// on a dedicated OS thread (e.g., via `std::thread::spawn`).
///
/// # Future Enhancement
/// Currently the server processes each command synchronously.  If response
/// time becomes critical, the inner handler can be moved to a `tokio` task.
pub fn start_pipe_server() {
    tracing::info!("Pipe server starting on {}", PIPE_NAME);

    // Convert the pipe name to a UTF-16 wide string required by Win32.
    let pipe_name: Vec<u16> = OsStr::new(PIPE_NAME)
        .encode_wide()
        .chain(Some(0))     // null-terminate
        .collect();

    loop {
        // ---- Create the pipe instance ----
        let handle = unsafe {
            CreateNamedPipeW(
                PCWSTR::from_raw(pipe_name.as_ptr()),
                PIPE_ACCESS_DUPLEX | FILE_FLAG_FIRST_PIPE_INSTANCE,
                PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT,
                PIPE_UNLIMITED_INSTANCES,
                512,   // output buffer size (small – commands are short)
                512,   // input buffer size
                0,     // default timeout (blocking)
                None,  // default security descriptor
            )
        };

        if handle == INVALID_HANDLE_VALUE {
            let err = unsafe { GetLastError() };
            tracing::error!("CreateNamedPipeW failed: {:?}", err);
            // Avoid a tight loop if the error is persistent.
            std::thread::sleep(std::time::Duration::from_secs(1));
            continue;
        }

        let pipe = PipeHandle(handle);

        // ---- Wait for the DLL client to connect ----
        unsafe {
            ConnectNamedPipe(pipe.raw(), None);
        }
        tracing::info!("Client connected to pipe");

        // ---- Service this client until the connection is lost ----
        loop {
            // 1. Read a complete framed message.
            let data = match read_frame(pipe.raw()) {
                Ok(bytes) => bytes,
                Err(e) => {
                    tracing::error!("Read error: {}", e);
                    break; // connection broken → wait for next client
                }
            };

            // 2. Deserialize the JSON payload.
            let cmd: CommandMessage = match serde_json::from_slice(&data) {
                Ok(c) => c,
                Err(e) => {
                    tracing::error!("Deserialization error: {}", e);
                    let resp = ResponseMessage {
                        status: ResponseStatus::Error,
                        message: format!("Invalid JSON: {}", e),
                        operation_id: None,
                        data: None,
                    };
                    let _ = write_frame(
                        pipe.raw(),
                        &serde_json::to_vec(&resp).unwrap_or_default(),
                    );
                    continue;
                }
            };

            // 3. Dispatch according to the command type.
            let response = match cmd {
                CommandMessage::ExecuteOperation(data) => {
                    tracing::info!("Received ExecuteOperation: {:?}", data);
                    // -----------------------------------------------------------
                    // TODO: Replace the placeholder below with the actual Use Case
                    // call once the Application Facade is wired in.
                    //
                    // Example:
                    //   let facade = get_facade_handle();  // obtain via AppState
                    //   let result = facade.execute_operation(command).await;
                    // -----------------------------------------------------------
                    let success = true; // ← placeholder
                    ResponseMessage {
                        status: if success {
                            ResponseStatus::Ok
                        } else {
                            ResponseStatus::Error
                        },
                        message: String::new(),
                        operation_id: None,
                        data: None,
                    }
                }
                CommandMessage::Ping => ResponseMessage {
                    status: ResponseStatus::Ok,
                    message: "pong".to_string(),
                    operation_id: None,
                    data: None,
                },
            };

            // 4. Send the response back to the DLL.
            let response_bytes =
                match serde_json::to_vec(&response) {
                    Ok(b) => b,
                    Err(e) => {
                        tracing::error!("Response serialization failed: {}", e);
                        break;
                    }
                };
            if let Err(e) = write_frame(pipe.raw(), &response_bytes) {
                tracing::error!("Write response failed: {}", e);
                break;
            }
            // Ensure the response is pushed to the wire before waiting for
            // the next command.
            unsafe { FlushFileBuffers(pipe.raw()).ok(); }
        }
    }
}