//! Data Transfer Objects used by inbound ports.
//! These are independent of domain entities and infrastructure.

mod operation_command;
mod operation_result;

pub use operation_command::{OperationCommand, OverwritePolicy};
pub use operation_result::OperationResult;