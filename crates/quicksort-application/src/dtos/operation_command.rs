//! Command DTO for executing file operations.
use serde::{Deserialize, Serialize};
use quicksort_domain::{FolderId, WindowsPath, OperationType};

/// Command to execute a file operation.
/// Created by adapters (GUI, CLI, Shell) and passed to ExecuteOperation port.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationCommand {
    pub operation_type: OperationType,
    pub source_paths: Vec<WindowsPath>,
    pub target_folder_id: Option<FolderId>, // None for Delete or Rename
    pub overwrite_policy: OverwritePolicy,
}

/// Conflict resolution strategy when a target file already exists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OverwritePolicy {
    /// Skip the file, log a warning.
    Skip,
    /// Replace the existing file.
    Overwrite,
    /// Append a suffix (e.g., "file (1).txt").
    AutoRename,
    /// Ask the user (handled by the ConflictResolver port).
    Ask,
}



