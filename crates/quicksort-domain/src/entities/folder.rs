//! Domain entity representing a user-defined folder.
//!
//! # Invariants
//! - The folder name must not be empty.
//! - The folder path must not be a root directory (e.g., `C:\`).
//! - `favorite` and `order` control visibility and sorting in the context menu.

use crate::{value_objects::{FolderId, WindowsPath}, errors::DomainError};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Statistics for a folder (how often it was used, last access time).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FolderStats {
    /// Total number of times files were moved/copied to this folder.
    pub use_count: u64,
    /// Timestamp of the last operation targeting this folder.
    pub last_used: Option<DateTime<Utc>>,
}

/// A user-defined folder that can be used as a target for file operations.
///
/// # Examples
/// ```rust
/// let folder = Folder::new("Documents", WindowsPath::new("C:\\Users\\Me\\Documents").unwrap());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Folder {
    /// Unique identifier for this folder.
    pub id: FolderId,
    /// Display name shown in the context menu and UI.
    pub name: String,
    /// Absolute path to the folder.
    pub path: WindowsPath,
    /// Whether this folder appears as a favorite in the context menu.
    #[serde(default)]
    pub favorite: bool,
    /// Sort order (lower values appear first). Used for menu ordering.
    #[serde(default)]
    pub order: u32,
    /// Usage statistics (not persisted if not needed, but available for analytics).
    #[serde(default)]
    pub stats: FolderStats,
    // OLD: pub created_at: SystemTime,
    /// When this folder was first created.
    pub created_at: DateTime<Utc>,
    // OLD: pub updated_at: SystemTime,
    /// When this folder was last modified.
    pub updated_at: DateTime<Utc>,
}

impl Folder {
    /// Creates a new folder with a generated ID and current timestamps.
    ///
    /// # Parameters
    /// - `name` – Display name (must not be empty).
    /// - `path` – Absolute filesystem path.
    pub fn new(name: impl Into<String>, path: WindowsPath) -> Self {
        let now = Utc::now();
        Self {
            id: FolderId::new(),
            name: name.into(),
            path,
            favorite: false,       // not a favorite by default
            order: 0,              // default sort order
            stats: Default::default(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Creates a new folder with an explicit ID (useful for testing or importing).
    pub fn with_id(id: FolderId, name: impl Into<String>, path: WindowsPath) -> Self {
        let now = Utc::now();
        Self {
            id,
            name: name.into(),
            path,
            favorite: false,
            order: 0,
            stats: Default::default(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Updates the folder name.
    ///
    /// # Errors
    /// Returns `DomainError::InvalidFolderName` if the name is empty.
    // OLD: returned `Result<(), String>` with a raw string error
    // NEW: returns `Result<(), DomainError>` for consistency
    pub fn update_name(&mut self, name: impl Into<String>) -> Result<(), DomainError> {
        let new_name = name.into();
        if new_name.trim().is_empty() {
            // OLD: return Err(format!("Folder name cannot be empty"));
            // NEW: use a proper domain error variant
            return Err(DomainError::InvalidFolderName);
        }
        self.name = new_name;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Updates the folder path.
    ///
    /// # Errors
    /// Returns `DomainError::IllegalDirectoryTarget` if the path is a root
    /// directory (e.g., `C:\`), which is too broad for a target folder.
    pub fn update_path(&mut self, new_path: WindowsPath) -> Result<(), DomainError> {
        if new_path.is_root() {
            return Err(DomainError::IllegalDirectoryTarget);
        }
        self.path = new_path;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Toggles the favorite status of this folder.
    ///
    /// When a folder is marked as favorite, it appears directly in the
    /// context menu's top-level list for quick access.
    pub fn toggle_favorite(&mut self) {
        self.favorite = !self.favorite;
        self.updated_at = Utc::now();
    }

    /// Records that this folder was used for an operation.
    ///
    /// Increments `use_count` and updates `last_used` to the current time.
    pub fn record_usage(&mut self) {
        self.stats.use_count += 1;
        self.stats.last_used = Some(Utc::now());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_path(path: &str) -> WindowsPath {
        WindowsPath::new(path).unwrap()
    }

    #[test]
    fn test_folder_new() {
        let f = Folder::new("Docs", test_path("C:\\Docs"));
        assert_eq!(f.name, "Docs");
        assert!(!f.favorite);
        assert_eq!(f.order, 0);
    }

    #[test]
    fn test_update_name() {
        let mut f = Folder::new("Docs", test_path("C:\\Docs"));
        f.update_name("Projects").unwrap();
        assert_eq!(f.name, "Projects");
    }

    #[test]
    fn test_update_name_empty_fails() {
        let mut f = Folder::new("Docs", test_path("C:\\Docs"));
        let result = f.update_name("");
        assert!(result.is_err());
    }

    #[test]
    fn test_update_path_root_fails() {
        let mut f = Folder::new("Docs", test_path("C:\\Docs"));
        let result = f.update_path(WindowsPath::new("C:\\").unwrap());
        assert!(matches!(result, Err(DomainError::IllegalDirectoryTarget)));
    }

    #[test]
    fn test_toggle_favorite() {
        let mut f = Folder::new("Docs", test_path("C:\\Docs"));
        assert!(!f.favorite);
        f.toggle_favorite();
        assert!(f.favorite);
        f.toggle_favorite();
        assert!(!f.favorite);
    }

    #[test]
    fn test_record_usage() {
        let mut f = Folder::new("Docs", test_path("C:\\Docs"));
        assert_eq!(f.stats.use_count, 0);
        assert!(f.stats.last_used.is_none());
        f.record_usage();
        assert_eq!(f.stats.use_count, 1);
        assert!(f.stats.last_used.is_some());
    }
}