use quicksort_ipc_contract::*;
use crate::pipe_client::error::PipeError;
use crate::pipe_client::transport::PipeTransport;
use crate::pipe_client::win32_transport::NamedPipeTransport;

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
        self.transport.write(&json)?;

        Ok(())
    }
}

/// Удобная функция для отправки команды через NamedPipeTransport.
pub fn send_command(command: &CommandMessage) -> Result<(), PipeError> {
    let mut client = PipeClient::new(NamedPipeTransport::new());
    client.send_command(command)
}