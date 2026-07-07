//! Transport abstraction.

use crate::pipe_client::error::PipeError;

pub trait PipeTransport: Send + Sync {
    fn connect(&mut self) -> Result<(), PipeError>;
    fn send(&mut self, data: &[u8]) -> Result<(), PipeError>;
    fn receive(&mut self) -> Result<Vec<u8>, PipeError>;
    fn disconnect(&mut self) -> Result<(), PipeError>;
}