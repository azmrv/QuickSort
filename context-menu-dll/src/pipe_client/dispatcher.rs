//! High-level dispatcher that hides transport details.

use crate::pipe_client::error::PipeError;
use crate::pipe_client::protocol::*;
use crate::pipe_client::transport::PipeTransport;
use crate::pipe_client::win32_transport::NamedPipeTransport;

/// Dispatches a command over the default named pipe transport.
pub fn send_action(action: PipeAction) -> Result<(), PipeError> {
    let command = PipeCommand {
        version: PROTOCOL_VERSION,
        action,
    };

    let mut transport = NamedPipeTransport::new();
    transport.connect()?;

    let json = serde_json::to_vec(&command)?;
    transport.write(&json)?;

    Ok(())
}

/// Convenience: send a Move operation.
pub fn send_move(
    source_paths: Vec<String>,
    target_folder_id: String,
    overwrite_policy: OverwritePolicy,
) -> Result<(), PipeError> {
    let action = PipeAction::ExecuteOperation {
        command: OperationCommand {
            operation_type: OperationType::Move,
            source_paths,
            target_folder_id: Some(target_folder_id),
            overwrite_policy,
        },
    };
    send_action(action)
}

/// Convenience: send a Copy operation.
pub fn send_copy(
    source_paths: Vec<String>,
    target_folder_id: String,
    overwrite_policy: OverwritePolicy,
) -> Result<(), PipeError> {
    let action = PipeAction::ExecuteOperation {
        command: OperationCommand {
            operation_type: OperationType::Copy,
            source_paths,
            target_folder_id: Some(target_folder_id),
            overwrite_policy,
        },
    };
    send_action(action)
}

/// Convenience: send a Delete operation.
pub fn send_delete(source_paths: Vec<String>) -> Result<(), PipeError> {
    let action = PipeAction::ExecuteOperation {
        command: OperationCommand {
            operation_type: OperationType::Delete,
            source_paths,
            target_folder_id: None,
            overwrite_policy: OverwritePolicy::Skip,
        },
    };
    send_action(action)
}

/// Convenience: send a Rename operation.
pub fn send_rename(source_path: String, new_name: String) -> Result<(), PipeError> {
    let action = PipeAction::ExecuteOperation {
        command: OperationCommand {
            operation_type: OperationType::Rename,
            source_paths: vec![source_path],
            target_folder_id: None,
            overwrite_policy: OverwritePolicy::Skip,
        },
    };
    send_action(action)
}