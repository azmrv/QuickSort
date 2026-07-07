//! Transport layer.

mod r#trait;
mod named_pipe;

pub use r#trait::PipeTransport;
pub use named_pipe::NamedPipeTransport;