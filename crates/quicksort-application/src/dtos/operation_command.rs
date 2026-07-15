//! Command DTO for executing file operations.
//!
//! This is the primary Data Transfer Object that travels from the adapters
//! (Tauri commands, CLI, IPC) into the Application Layer. It carries all the
//! information needed by `ExecuteOperationUseCase` to perform a Move, Copy,
//! Delete or Rename operation.
//!
//! # Design Decisions
//! - Uses domain types (`OperationType`, `WindowsPath`, `FolderId`) directly
//!   to avoid anemic DTOs and unnecessary mapping.  The Adapter layer is
//!   allowed to depend on domain value objects.
//! - All fields are public but the struct is not `Copy` – this prevents
//!   accidental duplication of commands that may lead to double-processing.
//! - `target_folder_id` and `target_paths` are mutually exclusive depending
//!   on the operation type:
//!   - Move/Copy → `target_folder_id` is required, `target_paths` is `None`.
//!   - Rename → `target_paths` is required, `target_folder_id` is `None`.
//!   - Delete → both are `None`.

use serde::{Deserialize, Serialize};
use quicksort_domain::{FolderId, WindowsPath, OperationType};

/// Command to execute a file operation.
///
/// Created by adapters (GUI, CLI, Shell) and passed to `ExecuteOperation`
/// port. The use case validates the command, resolves conflicts according
/// to the chosen policy, and performs the actual file operations.
///
/// # Examples
/// ```rust
/// let cmd = OperationCommand::new_move(
///     vec![source_path],
///     target_folder_id,
///     OverwritePolicy::Skip,
/// );
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationCommand {
    /// The type of operation to perform.
    pub operation_type: OperationType,

    /// Absolute paths of the files or directories to operate on.
    /// Must contain at least one entry.
    pub source_paths: Vec<WindowsPath>,

    /// Target folder ID – used for Move/Copy operations where the
    /// destination is a container. Must be `Some` for Move/Copy,
    /// `None` for Rename/Delete.
    pub target_folder_id: Option<FolderId>,

    /// Explicit list of target paths – required for Rename operations.
    /// For Move/Copy this field must be `None`.
    /// When present, its length must match `source_paths`.
    pub target_paths: Option<Vec<WindowsPath>>,

    /// Conflict resolution strategy when a target file already exists.
    pub overwrite_policy: OverwritePolicy,
}

impl OperationCommand {
    /// Creates a new Move command.
    ///
    /// # Panics (in debug builds)
    /// If `source_paths` is empty or `target_folder_id` is `None`.
    pub fn new_move(
        source_paths: Vec<WindowsPath>,
        target_folder_id: FolderId,
        policy: OverwritePolicy,
    ) -> Self {
        debug_assert!(!source_paths.is_empty(), "source_paths must not be empty");
        Self {
            operation_type: OperationType::Move,
            source_paths,
            target_folder_id: Some(target_folder_id),
            target_paths: None,
            overwrite_policy: policy,
        }
    }

    /// Creates a new Copy command.
    pub fn new_copy(
        source_paths: Vec<WindowsPath>,
        target_folder_id: FolderId,
        policy: OverwritePolicy,
    ) -> Self {
        debug_assert!(!source_paths.is_empty(), "source_paths must not be empty");
        Self {
            operation_type: OperationType::Copy,
            source_paths,
            target_folder_id: Some(target_folder_id),
            target_paths: None,
            overwrite_policy: policy,
        }
    }

    /// Creates a new Delete command.
    pub fn new_delete(source_paths: Vec<WindowsPath>) -> Self {
        debug_assert!(!source_paths.is_empty(), "source_paths must not be empty");
        Self {
            operation_type: OperationType::Delete,
            source_paths,
            target_folder_id: None,
            target_paths: None,
            overwrite_policy: OverwritePolicy::Skip, // not relevant for Delete
        }
    }

    /// Creates a new Rename command.
    ///
    /// # Panics (in debug builds)
    /// If `source_paths` and `target_paths` have different lengths.
    pub fn new_rename(
        source_paths: Vec<WindowsPath>,
        target_paths: Vec<WindowsPath>,
        policy: OverwritePolicy,
    ) -> Self {
        debug_assert_eq!(
            source_paths.len(),
            target_paths.len(),
            "source and target path counts must match"
        );
        Self {
            operation_type: OperationType::Rename,
            source_paths,
            target_folder_id: None,
            target_paths: Some(target_paths),
            overwrite_policy: policy,
        }
    }
}

/// Conflict resolution strategy when a target file already exists.
///
/// This policy controls how the `ExecuteOperationUseCase` handles
/// destination collisions during Move and Copy operations.
/// For Delete and Rename the policy is irrelevant and ignored.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OverwritePolicy {
    /// Skip the conflicting file and return a `UseCaseError::Conflict`.
    /// The source file is left untouched.
    Skip,

    /// Replace the existing file with the new one.
    /// **Warning:** This is destructive and cannot be undone.
    Overwrite,

    /// Automatically generate a unique name by appending a numeric suffix
    /// (e.g., "file (1).txt", "file (2).txt").  The original destination
    /// file is preserved.
    AutoRename,

    /// Prompt the user for a decision.  When called from a non-interactive
    /// context (e.g., IPC from the shell extension DLL), this policy
    /// falls back to `AutoRename`.
    Ask,
}