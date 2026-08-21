//! Domain entities - core business objects

mod conflict_resolution;
mod duplicate_check;
mod folder;
mod operation;
mod settings;

pub use crate::value_objects::{FolderId, OperationId};
pub use conflict_resolution::{ConflictContext, ConflictResolution, ConflictStats};
pub use duplicate_check::{DuplicateCheckMode, DuplicateCheckResult, DuplicateChecker};
pub use folder::Folder;
pub use operation::{Operation, OperationState, OperationType};
pub use settings::{DefaultOperation, DefaultOverwritePolicy, DuplicateCheckConfig, Settings};
