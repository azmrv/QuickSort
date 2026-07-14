//! Domain entities - core business objects

mod folder;
mod operation;

pub use folder::Folder;
pub use operation::{Operation, OperationState, OperationType};
pub use crate::value_objects::{FolderId, OperationId};