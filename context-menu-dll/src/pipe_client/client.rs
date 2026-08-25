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

    pub fn send_command(&mut self, command: &CommandMessage) -> Result<ResponseMessage, PipeError> {
        self.transport.connect()?;

        let json = serde_json::to_vec(command)?;

        let len = json.len() as u32;
        let len_bytes = len.to_le_bytes();
        let mut framed = Vec::with_capacity(4 + json.len());
        framed.extend_from_slice(&len_bytes);
        framed.extend_from_slice(&json);
        self.transport.send(&framed)?;

        // transport.receive() already reads the [u32 LE length][payload] frame
        // and returns only the payload bytes.
        let response_bytes = self.transport.receive()?;
        let resp: ResponseMessage = serde_json::from_slice(&response_bytes)?;
        self.transport.disconnect()?;
        Ok(resp)
    }
}

pub fn send_command(command: &CommandMessage) -> Result<ResponseMessage, PipeError> {
    let mut client = PipeClient::new(NamedPipeTransport::new());
    client.send_command(command)
}

pub fn move_to_folder(
    sources: Vec<String>,
    target_folder_id: String,
    overwrite_policy: OverwritePolicy,
) -> Result<ResponseMessage, PipeError> {
    let cmd = CommandMessage::ExecuteOperation(ExecuteOperationData {
        operation_type: OperationType::Move,
        source_paths: sources,
        target_folder_id: Some(target_folder_id),
        overwrite_policy,
    });
    send_command(&cmd)
}

/// Sends a SelectFolder command to open the folder selector UI.
///
/// The server will show/focus the main window and emit a `pending-file`
/// event so the frontend displays the SelectorPage.
pub fn select_folder(
    source_paths: Vec<String>,
) -> Result<ResponseMessage, PipeError> {
    let cmd = CommandMessage::SelectFolder(SelectFolderData { source_paths });
    send_command(&cmd)
}

#[allow(dead_code)]
pub fn ping() -> Result<ResponseMessage, PipeError> {
    send_command(&CommandMessage::Ping)
}
