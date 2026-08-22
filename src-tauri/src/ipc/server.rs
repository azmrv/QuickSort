//! Named Pipe server that receives commands from the shell extension DLL.
//!
//! This module runs in a dedicated background thread and listens for
//! incoming connections on `\\.\pipe\quicksort_cmd`.  Every command
//! received is deserialized, validated, and forwarded to the Application
//! Facade for execution.  A response is sent back to the client (DLL).

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::sync::Arc;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{CloseHandle, GetLastError, HANDLE, INVALID_HANDLE_VALUE};
use windows::Win32::Storage::FileSystem::{
    FlushFileBuffers, FILE_FLAG_FIRST_PIPE_INSTANCE, PIPE_ACCESS_DUPLEX,
};
use windows::Win32::System::Pipes::{
    ConnectNamedPipe, CreateNamedPipeW, PIPE_READMODE_BYTE, PIPE_TYPE_BYTE,
    PIPE_UNLIMITED_INSTANCES, PIPE_WAIT,
};

use quicksort_application::{
    ApplicationFacadeImpl, DuplicateCheckMode, ExecuteOperation, FolderId, OperationCommand,
    OperationType as DomainOpType, OverwritePolicy as AppOverwritePolicy, WindowsPath,
};
use quicksort_ipc_contract::{
    CommandMessage, ExecuteOperationData, OperationType as IpcOpType,
    OverwritePolicy as IpcOverwritePolicy, ResponseMessage, ResponseStatus,
};

use super::framing::{read_frame, write_frame};

const PIPE_NAME: &str = r"\\.\pipe\quicksort_cmd";

// ---------------------------------------------------------------------------
// RAII wrapper for HANDLE
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Type conversions: IPC contract → Application DTOs
// ---------------------------------------------------------------------------

fn convert_operation_type(ty: IpcOpType) -> DomainOpType {
    match ty {
        IpcOpType::Move => DomainOpType::Move,
        IpcOpType::Copy => DomainOpType::Copy,
        IpcOpType::Delete => DomainOpType::Delete,
        IpcOpType::Rename => DomainOpType::Rename,
    }
}

fn convert_overwrite_policy(p: IpcOverwritePolicy) -> AppOverwritePolicy {
    match p {
        IpcOverwritePolicy::Skip => AppOverwritePolicy::Skip,
        IpcOverwritePolicy::Overwrite => AppOverwritePolicy::Overwrite,
        IpcOverwritePolicy::AutoRename => AppOverwritePolicy::AutoRename,
        IpcOverwritePolicy::Ask => AppOverwritePolicy::AutoRename, // non-interactive fallback
    }
}

fn convert_execute_data(data: ExecuteOperationData) -> Option<OperationCommand> {
    let source_paths: Vec<WindowsPath> = data
        .source_paths
        .iter()
        .filter_map(|p| WindowsPath::new(p).ok())
        .collect();

    if source_paths.is_empty() {
        return None;
    }

    let target_folder_id = data
        .target_folder_id
        .and_then(|id| FolderId::from_string(&id).ok());

    Some(OperationCommand {
        operation_type: convert_operation_type(data.operation_type),
        source_paths,
        target_folder_id,
        target_paths: None,
        overwrite_policy: convert_overwrite_policy(data.overwrite_policy),
        duplicate_check_mode: DuplicateCheckMode::default(),
    })
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Starts the pipe server loop.
///
/// # Blocking
/// This function never returns under normal operation.  It must be spawned
/// on a dedicated OS thread.
pub fn start_pipe_server(facade: Arc<ApplicationFacadeImpl>) {
    tracing::info!("Pipe server starting on {}", PIPE_NAME);

    let pipe_name: Vec<u16> = OsStr::new(PIPE_NAME).encode_wide().chain(Some(0)).collect();

    // Create a Tokio runtime for blocking on async facade calls.
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to create tokio runtime for pipe server");

    loop {
        let handle = unsafe {
            CreateNamedPipeW(
                PCWSTR::from_raw(pipe_name.as_ptr()),
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
            tracing::error!("CreateNamedPipeW failed: {:?}", err);
            std::thread::sleep(std::time::Duration::from_secs(1));
            continue;
        }

        let pipe = PipeHandle(handle);

        unsafe {
            let _ = ConnectNamedPipe(pipe.raw(), None);
        }
        tracing::info!("Client connected to pipe");

        loop {
            let data = match read_frame(pipe.raw()) {
                Ok(bytes) => bytes,
                Err(e) => {
                    // ERROR_BROKEN_PIPE (0x8007006D) is expected when the DLL
                    // client disconnects after sending a single command.
                    if e.contains("0x8007006D") || e.contains("broken pipe") {
                        tracing::debug!("Client disconnected: {}", e);
                    } else {
                        tracing::error!("Read error: {}", e);
                    }
                    break;
                }
            };

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
                    let _ = write_frame(pipe.raw(), &serde_json::to_vec(&resp).unwrap_or_default());
                    continue;
                }
            };

            let response = match cmd {
                CommandMessage::ExecuteOperation(data) => {
                    tracing::info!("Received ExecuteOperation: {:?}", data);

                    let response = match convert_execute_data(data) {
                        Some(command) => match rt.block_on(facade.execute(command)) {
                            Ok(result) => {
                                let op_id = result.operation_id.to_string();
                                let processed = result.processed_files;
                                ResponseMessage {
                                    status: ResponseStatus::Ok,
                                    message: format!("Processed {} files", processed),
                                    operation_id: Some(op_id),
                                    data: None,
                                }
                            }
                            Err(e) => ResponseMessage {
                                status: ResponseStatus::Error,
                                message: e.to_string(),
                                operation_id: None,
                                data: None,
                            },
                        },
                        None => ResponseMessage {
                            status: ResponseStatus::Error,
                            message: "Invalid command: no valid source paths".to_string(),
                            operation_id: None,
                            data: None,
                        },
                    };
                    response
                }
                CommandMessage::Ping => ResponseMessage {
                    status: ResponseStatus::Ok,
                    message: "pong".to_string(),
                    operation_id: None,
                    data: None,
                },
            };

            let response_bytes = match serde_json::to_vec(&response) {
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
            unsafe {
                FlushFileBuffers(pipe.raw()).ok();
            }
        }
    }
}
