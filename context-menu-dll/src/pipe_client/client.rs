//! High-level IPC client.

use crate::pipe_client::error::PipeError;
use crate::pipe_client::protocol::*;
use crate::pipe_client::transport::{NamedPipeTransport, PipeTransport};

pub struct PipeClient<T: PipeTransport> {
    transport: T,
    timeout_ms: u64,
}

impl<T: PipeTransport> PipeClient<T> {
    pub fn new(transport: T) -> Self {
        Self {
            transport,
            timeout_ms: 1000,
        }
    }

    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    pub fn send_command(&mut self, command: CommandMessage) -> Result<ResponseMessage, PipeError> {
        let bytes = Codec::encode_command(command)?;

        self.transport.connect()?;
        self.transport.send(&bytes)?;

        let response_bytes = self.transport.receive()?;
        let response = Codec::decode_response(&response_bytes)?;

        self.transport.disconnect()?;

        match response.status {
            ResponseStatus::Ok => Ok(response),
            ResponseStatus::Error => Err(PipeError::OperationFailed(response.message)),
            ResponseStatus::Pending => Ok(response),
        }
    }
}

impl Default for PipeClient<NamedPipeTransport> {
    fn default() -> Self {
        Self::new(NamedPipeTransport::new())
    }
}

/// High-level API for Shell Extension.
pub fn move_to_folder(
    source_paths: Vec<String>,
    target_folder_id: String,
    overwrite_policy: OverwritePolicy,
) -> Result<(), PipeError> {
    let mut client = PipeClient::default();

    let data = ExecuteOperationData {
        operation_type: OperationType::Move,
        source_paths,
        target_folder_id: Some(target_folder_id),
        overwrite_policy,
    };
    let command = CommandMessage::ExecuteOperation(data);

    let response = client.send_command(command)?;

    if response.status == ResponseStatus::Error {
        return Err(PipeError::OperationFailed(response.message));
    }

    Ok(())
}

pub fn ping() -> Result<(), PipeError> {
    let mut client = PipeClient::default();
    let command = CommandMessage::Ping;
    let response = client.send_command(command)?;
    if response.status == ResponseStatus::Error {
        return Err(PipeError::OperationFailed(response.message));
    }
    Ok(())
}