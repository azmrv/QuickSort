//! Folder metadata DTO for UI display.
//!
//! Carries summary information about a folder (or a single item) from the
//! file system adapter to the frontend, avoiding a separate metadata call per
//! file in large trees.

use serde::{Deserialize, Serialize};

/// Summary metadata about a folder for the UI.
///
/// A path that does not exist yields `exists=false` with zeroed counters
/// instead of an error, so the frontend can render it as unavailable without
/// special-casing error handling.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FolderMetadata {
    /// Whether the path exists on disk.
    pub exists: bool,
    /// Whether the path is a directory (false for files and symlinks).
    pub is_dir: bool,
    /// Total size in bytes of all files inside the folder tree.
    pub total_size: u64,
    /// Number of files inside the folder tree (directories excluded).
    pub item_count: u64,
}
