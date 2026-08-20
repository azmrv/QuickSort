use crate::pipe_client::error::PipeError;
use crate::pipe_client::transport::named_pipe::NamedPipeTransport;
use crate::pipe_client::transport::PipeTransport;
use quicksort_ipc_contract::*;

pub struct PipeClient<T: PipeTransport> {
    transport: T,
}

impl<T: PipeTransport> PipeClient<T> {
    pub fn new(transport: T) -> Self {
        Self { transport }
    }

    pub fn send_command(&mut self, command: &CommandMessage) -> Result<(), PipeError> {
        self.transport.connect()?;

        let json = serde_json::to_vec(command)?;
        self.transport.send(&json)?;

        Ok(())
    }
}

/// Sends a command through a new NamedPipeTransport connection.
pub fn send_command(command: &CommandMessage) -> Result<(), PipeError> {
    let mut client = PipeClient::new(NamedPipeTransport::new());
    client.send_command(command)
}

/// Moves source files to a target folder via IPC.
pub fn move_to_folder(
    sources: Vec<String>,
    target_folder_id: String,
    overwrite_policy: OverwritePolicy,
) -> Result<(), PipeError> {
    let cmd = CommandMessage::ExecuteOperation(ExecuteOperationData {
        operation_type: OperationType::Move,
        source_paths: sources,
        target_folder_id: Some(target_folder_id),
        overwrite_policy,
    });
    send_command(&cmd)
}

/// Sends a Ping command to check server availability.
pub fn ping() -> Result<(), PipeError> {
    send_command(&CommandMessage::Ping)
}
