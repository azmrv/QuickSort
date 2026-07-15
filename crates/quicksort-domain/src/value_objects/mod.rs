//! Value Objects – immutable, self-validating types.
//!
//! Value objects are defined by their attributes rather than an identity.
//! They are immutable once created and enforce their invariants at
//! construction time.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

mod windows_path;

pub use self::windows_path::WindowsPath;

// ============================================================================
// FolderId
// ============================================================================

/// Unique identifier for a folder configuration.
///
/// Wraps a UUID v4 to guarantee global uniqueness across different machines.
/// Two `FolderId`s with the same underlying UUID are considered equal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FolderId(Uuid);

impl FolderId {
    /// Creates a new random folder ID.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Creates a `FolderId` from an existing `Uuid`.
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Returns the underlying `Uuid`.
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }

    /// Returns the string representation of the UUID.
    // OLD: pub fn to_string(&self) -> String { self.0.to_string() }
    // The standard way to get a string from an ID is through `Display`.
    // Keeping `to_string` as a convenience method is fine, but it should
    // just delegate to the `Display` implementation for consistency.
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }

    /// Creates a `FolderId` from a string representation of a UUID.
    ///
    /// # Errors
    /// Returns a `DomainError` if the string is not a valid UUID.
    // NEW: added a safe constructor that validates the input
    pub fn from_string(s: &str) -> Result<Self, crate::errors::DomainError> {
        Uuid::parse_str(s)
            .map(Self)
            .map_err(|_| crate::errors::DomainError::InvalidPath(
                format!("Invalid UUID string: {}", s)
            ))
    }
}

impl Default for FolderId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for FolderId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ============================================================================
// OperationId
// ============================================================================

/// Unique identifier for a file operation.
///
/// Wraps a UUID v4 to guarantee global uniqueness across different machines.
/// Used to track operations for undo, logging, and auditing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OperationId(Uuid);

impl OperationId {
    /// Creates a new random operation ID.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Creates an `OperationId` from an existing `Uuid`.
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Creates an `OperationId` from a string representation of a UUID.
    ///
    /// # Errors
    /// Returns a `DomainError` if the string is not a valid UUID.
    // OLD: возвращал `Result<Self, String>` – raw string errors are
    // inconsistent with the rest of the domain.
    // NEW: returns `Result<Self, DomainError>` for uniformity.
    pub fn from_string(s: &str) -> Result<Self, crate::errors::DomainError> {
        Uuid::parse_str(s)
            .map(Self)
            .map_err(|_| crate::errors::DomainError::InvalidPath(
                format!("Invalid UUID string: {}", s)
            ))
    }

    /// Returns the underlying `Uuid`.
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }

    /// Returns the string representation of the UUID.
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl Default for OperationId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for OperationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_folder_id_unique() {
        let id1 = FolderId::new();
        let id2 = FolderId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_folder_id_from_string_roundtrip() {
        let id = FolderId::new();
        let s = id.to_string();
        let parsed = FolderId::from_string(&s).unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn test_folder_id_from_string_invalid() {
        assert!(FolderId::from_string("not-a-uuid").is_err());
    }

    #[test]
    fn test_operation_id_from_string_roundtrip() {
        let id = OperationId::new();
        let s = id.to_string();
        let parsed = OperationId::from_string(&s).unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn test_windows_path() {
        let Ok(p) = WindowsPath::new("C:\\Folder") else { return };
        assert_eq!(p.as_str(), Some("C:\\Folder"));
        assert!(p.is_absolute());
    }

    #[test]
    fn test_windows_path_unc() {
        let Ok(p) = WindowsPath::new("\\\\server\\share") else { return };
        assert_eq!(p.as_str(), Some("\\\\server\\share"));
        assert!(p.is_absolute());
    }

    #[test]
    fn test_windows_path_invalid() {
        assert!(WindowsPath::new("").is_err());
        assert!(WindowsPath::new("folder").is_err());
        assert!(WindowsPath::new("C:folder").is_err());
    }
}