mod error;
pub mod protocol;
pub mod transport;
pub mod client;

pub use error::PipeError;
pub use protocol::codec::{CommandKind, OperationType, OverwritePolicy, ResponseMessage, ResponseStatus};
pub use protocol::envelope::MessageEnvelope;
pub use protocol::header::{MAGIC, PROTOCOL_VERSION, MAX_MESSAGE_SIZE, MessageHeader};
pub use transport::named_pipe::NamedPipeTransport;
pub use transport::trait::PipeTransport;
pub use client::{PipeClient, move_to_folder, ping};