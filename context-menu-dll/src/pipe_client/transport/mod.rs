//! Transport layer.

pub(crate) mod named_pipe;
pub(crate) mod pipe_trait;

pub use pipe_trait::PipeTransport;
