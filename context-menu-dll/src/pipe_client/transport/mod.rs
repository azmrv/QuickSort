//! Transport layer.

pub(crate) mod pipe_trait;
pub(crate) mod named_pipe;

pub use pipe_trait::PipeTransport;
pub use named_pipe::NamedPipeTransport;