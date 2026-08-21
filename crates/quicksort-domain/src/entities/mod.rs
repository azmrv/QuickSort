//! Domain entities - core business objects

mod folder;
mod operation;
mod settings;

pub use crate::value_objects::{FolderId, OperationId};
pub use folder::Folder;
pub use operation::{Operation, OperationState, OperationType};
pub use settings::{
    DefaultOperation, DefaultOverwritePolicy, DuplicateCheckConfig, DuplicateCheckMode, Settings,
};
