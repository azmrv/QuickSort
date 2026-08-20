//! Domain entity representing a user-defined folder.
//!
//! # Invariants
//! - The folder name must not be empty.
//! - The folder path must not be a root directory (e.g., `C:\`).
//! - `favorite` and `order` control visibility and sorting in the context menu.

use crate::{
    errors::DomainError,
    value_objects::{FolderId, WindowsPath},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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
/// use quicksort_domain::entities::Folder;
/// use quicksort_domain::value_objects::WindowsPath;
/// let folder = Folder::new("Documents", WindowsPath::new("C:\\Users\\Me\\Documents").unwrap()).unwrap();
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
    // When this folder was first created.
    pub created_at: DateTime<Utc>,
    // When this folder was last modified.
    pub updated_at: DateTime<Utc>,
}

impl Folder {
    /// Creates a new folder with a generated ID and current timestamps.
    ///
    /// # Parameters
    /// - `name` – Display name (must not be empty, max 100 chars, no special chars).
    /// - `path` – Absolute filesystem path.
    ///
    /// # Errors
    /// Returns `DomainError::InvalidFolderName` if name is empty or contains
    /// forbidden characters (`\`, `/`, `:`, `*`, `?`, `"`, `<`, `>`, `|`).
    pub fn new(name: impl Into<String>, path: WindowsPath) -> Result<Self, DomainError> {
        let name_str = name.into();
        Self::validate_name(&name_str)?;
        let now = Utc::now();
        Ok(Self {
            id: FolderId::new(),
            name: name_str,
            path,
            favorite: false,
            order: 0,
            stats: Default::default(),
            created_at: now,
            updated_at: now,
        })
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
    // returns `Result<(), DomainError>` for consistency
    pub fn update_name(&mut self, name: impl Into<String>) -> Result<(), DomainError> {
        let new_name = name.into();
        if new_name.trim().is_empty() {
            // use a proper domain error variant
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

    /// Validates a folder name against security and usability rules.
    ///
    /// # Rules
    /// - Must not be empty or whitespace only.
    /// - Max 100 characters.
    /// - No Windows-forbidden characters: `\ / : * ? " < > |`
    /// - No control characters (ASCII 0-31).
    fn validate_name(name: &str) -> Result<(), DomainError> {
        if name.trim().is_empty() {
            return Err(DomainError::InvalidFolderName);
        }
        if name.len() > 100 {
            return Err(DomainError::InvalidFolderName);
        }
        // Forbidden characters for Windows file/folder names
        const FORBIDDEN: &[char] = &['\\', '/', ':', '*', '?', '"', '<', '>', '|'];
        if name.chars().any(|c| FORBIDDEN.contains(&c)) {
            return Err(DomainError::InvalidFolderName);
        }
        // Reject control characters (ASCII 0-31)
        if name.chars().any(|c| c.is_control()) {
            return Err(DomainError::InvalidFolderName);
        }
        Ok(())
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
        let f = Folder::new("Docs", test_path("C:\\Docs")).unwrap();
        assert_eq!(f.name, "Docs");
        assert!(!f.favorite);
        assert_eq!(f.order, 0);
    }

    #[test]
    fn test_folder_new_empty_name_fails() {
        let result = Folder::new("", test_path("C:\\Docs"));
        assert!(result.is_err());
        assert!(matches!(result, Err(DomainError::InvalidFolderName)));
    }

    #[test]
    fn test_folder_new_forbidden_chars_fails() {
        let result = Folder::new("Docs\\Invalid", test_path("C:\\Docs"));
        assert!(result.is_err());
    }

    #[test]
    fn test_folder_new_too_long_fails() {
        let long_name = "A".repeat(101);
        let result = Folder::new(long_name, test_path("C:\\Docs"));
        assert!(result.is_err());
    }

    #[test]
    fn test_folder_new_control_chars_fails() {
        let result = Folder::new("Docs\x00Invalid", test_path("C:\\Docs"));
        assert!(result.is_err());
    }

    #[test]
    fn test_update_name() {
        let mut f = Folder::new("Docs", test_path("C:\\Docs")).unwrap();
        f.update_name("Projects").unwrap();
        assert_eq!(f.name, "Projects");
    }

    #[test]
    fn test_update_name_empty_fails() {
        let mut f = Folder::new("Docs", test_path("C:\\Docs")).unwrap();
        let result = f.update_name("");
        assert!(result.is_err());
    }

    #[test]
    fn test_update_path_root_fails() {
        let mut f = Folder::new("Docs", test_path("C:\\Docs")).unwrap();
        let result = f.update_path(WindowsPath::new("C:\\").unwrap());
        assert!(matches!(result, Err(DomainError::IllegalDirectoryTarget)));
    }

    #[test]
    fn test_toggle_favorite() {
        let mut f = Folder::new("Docs", test_path("C:\\Docs")).unwrap();
        assert!(!f.favorite);
        f.toggle_favorite();
        assert!(f.favorite);
        f.toggle_favorite();
        assert!(!f.favorite);
    }

    #[test]
    fn test_record_usage() {
        let mut f = Folder::new("Docs", test_path("C:\\Docs")).unwrap();
        assert_eq!(f.stats.use_count, 0);
        assert!(f.stats.last_used.is_none());
        f.record_usage();
        assert_eq!(f.stats.use_count, 1);
        assert!(f.stats.last_used.is_some());
    }
}
