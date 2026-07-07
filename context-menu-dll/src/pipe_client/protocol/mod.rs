//! IPC protocol definitions.

pub(crate) mod header;
pub(crate) mod envelope;
pub(crate) mod codec;

pub use header::*;
pub use envelope::*;
pub use codec::*;