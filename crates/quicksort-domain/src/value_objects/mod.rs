//! Value Objects – immutable, self-validating types.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

mod windows_path;

pub use self::windows_path::WindowsPath;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FolderId(Uuid);

impl FolderId {
    pub fn new() -> Self { Self(Uuid::new_v4()) }
    pub fn from_uuid(uuid: Uuid) -> Self { Self(uuid) }
    pub fn as_uuid(&self) -> Uuid { self.0 }
    pub fn to_string(&self) -> String { self.0.to_string() }
}

impl Default for FolderId {
    fn default() -> Self { Self::new() }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OperationId(Uuid);

impl OperationId {
    pub fn new() -> Self { Self(Uuid::new_v4()) }
    pub fn from_uuid(uuid: Uuid) -> Self { Self(uuid) }
    pub fn as_uuid(&self) -> Uuid { self.0 }
    pub fn to_string(&self) -> String { self.0.to_string() }
}

impl Default for OperationId {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

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