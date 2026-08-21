//! Domain entities - core business objects

mod duplicate_check;
mod folder;
mod operation;
mod settings;

pub use crate::value_objects::{FolderId, OperationId};
pub use duplicate_check::{DuplicateCheckMode, DuplicateCheckResult, DuplicateChecker};
pub use folder::Folder;
pub use operation::{Operation, OperationState, OperationType};
pub use settings::{DefaultOperation, DefaultOverwritePolicy, DuplicateCheckConfig, Settings};
