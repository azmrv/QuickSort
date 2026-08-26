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
    ApplicationFacadeImpl, DuplicateCheckMode, ExecuteOperation, FolderId, GetFolders,
    OperationCommand, OperationType as DomainOpType, OverwritePolicy as AppOverwritePolicy,
    WindowsPath,
};
use quicksort_ipc_contract::{
    CommandMessage, ExecuteOperationData, OperationType as IpcOpType,
    OverwritePolicy as IpcOverwritePolicy, ResponseMessage, ResponseStatus, SelectFolderData,
};

use tauri::{Emitter, Manager};

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

/// Resolves a raw `target_folder_path` to a registered folder ID.
///
/// Called when `target_folder_id` is `None` but `target_folder_path` is
/// `Some` — the DLL sends a raw path (e.g. from the "ChoosePath" dialog)
/// and the server must find the matching registered folder.
async fn resolve_target_folder_path(
    path: &str,
    facade: &ApplicationFacadeImpl,
) -> Option<FolderId> {
    let folders = facade.get_all().await.ok()?;
    folders.iter().find(|f| f.path.to_string() == path).map(|f| f.id)
}

// ---------------------------------------------------------------------------
// SelectFolder handler
// ---------------------------------------------------------------------------

/// Payload for the `pending-file` Tauri event.
#[derive(Clone, serde::Serialize)]
struct PendingFilePayload {
    files: Vec<String>,
}

/// Handles a `SelectFolder` command from the DLL.
///
/// Stores all source file paths as pending, shows/focuses the main window,
/// and emits a `pending-file` event so the frontend displays the SelectorPage.
fn handle_select_folder(data: SelectFolderData) -> ResponseMessage {
    if data.source_paths.is_empty() {
        return ResponseMessage {
            status: ResponseStatus::Error,
            message: "No source files provided".to_string(),
            operation_id: None,
            data: None,
        };
    }

    // Store all file paths for the frontend.
    crate::pending::set_pending_files(data.source_paths.clone());

    match crate::ipc::get_app_handle() {
        Some(app) => {
            // Show and focus the main window.
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }

            // Emit event so the frontend switches to selector mode.
            let _ = app.emit(
                "pending-file",
                PendingFilePayload {
                    files: data.source_paths.clone(),
                },
            );

            tracing::info!(
                total_files = data.source_paths.len(),
                "SelectFolder: window shown, event emitted"
            );

            ResponseMessage {
                status: ResponseStatus::Ok,
                message: "Folder selector opened".to_string(),
                operation_id: None,
                data: None,
            }
        }
        None => {
            tracing::error!("SelectFolder: AppHandle not available");
            ResponseMessage {
                status: ResponseStatus::Error,
                message: "App not initialized".to_string(),
                operation_id: None,
                data: None,
            }
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

                    // Resolve target_folder_path to a registered folder ID
                    // when target_folder_id is not provided.
                    let mut data = data;
                    let mut early_response: Option<ResponseMessage> = None;

                    if data.target_folder_id.is_none() {
                        if let Some(ref path) = data.target_folder_path.clone() {
                            match rt.block_on(resolve_target_folder_path(path, &facade)) {
                                Some(folder_id) => {
                                    tracing::info!(
                                        "Resolved target_folder_path '{}' to folder ID '{}'",
                                        path,
                                        folder_id
                                    );
                                    data.target_folder_id = Some(folder_id.to_string());
                                }
                                None => {
                                    tracing::error!(
                                        "Target folder path '{}' not found in registered folders",
                                        path
                                    );
                                    early_response = Some(ResponseMessage {
                                        status: ResponseStatus::Error,
                                        message: format!("Target folder not found: {}", path),
                                        operation_id: None,
                                        data: None,
                                    });
                                }
                            }
                        } else {
                            early_response = Some(ResponseMessage {
                                status: ResponseStatus::Error,
                                message: "Move/Copy requires target_folder_id or target_folder_path"
                                    .to_string(),
                                operation_id: None,
                                data: None,
                            });
                        }
                    }

                    match early_response {
                        Some(resp) => resp,
                        None => {
                            let response = match convert_execute_data(data) {
                                Some(command) => match rt.block_on(facade.execute(command)) {
                                    Ok(result) => {
                                        let op_id = result.operation_id.to_string();
                                        let processed = result.processed_files;
                                        tracing::info!(
                                            "ExecuteOperation OK: op_id={}, files={}, bytes={}",
                                            op_id,
                                            processed,
                                            result.bytes_moved
                                        );
                                        ResponseMessage {
                                            status: ResponseStatus::Ok,
                                            message: format!("Processed {} files", processed),
                                            operation_id: Some(op_id),
                                            data: None,
                                        }
                                    }
                                    Err(e) => {
                                        tracing::error!("ExecuteOperation FAIL: {}", e);
                                        ResponseMessage {
                                            status: ResponseStatus::Error,
                                            message: e.to_string(),
                                            operation_id: None,
                                            data: None,
                                        }
                                    }
                                },
                                None => {
                                    tracing::error!("ExecuteOperation FAIL: no valid source paths");
                                    ResponseMessage {
                                        status: ResponseStatus::Error,
                                        message: "Invalid command: no valid source paths".to_string(),
                                        operation_id: None,
                                        data: None,
                                    }
                                }
                            };
                            response
                        }
                    }
                }
                CommandMessage::Ping => ResponseMessage {
                    status: ResponseStatus::Ok,
                    message: "pong".to_string(),
                    operation_id: None,
                    data: None,
                },
                CommandMessage::SelectFolder(data) => {
                    tracing::info!("Received SelectFolder: {:?}", data);
                    handle_select_folder(data)
                }
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
