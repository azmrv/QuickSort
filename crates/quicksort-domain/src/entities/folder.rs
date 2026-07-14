//! Domain entity representing a user-defined folder.

use crate::{value_objects::{FolderId, WindowsPath}, errors::DomainError};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Folder {
    pub id: FolderId,
    pub name: String,
    pub path: WindowsPath,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}

impl Folder {
    pub fn new(name: impl Into<String>, path: WindowsPath) -> Self {
        Self {
            id: FolderId::new(),
            name: name.into(),
            path,
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
        }
    }

    pub fn with_id(id: FolderId, name: impl Into<String>, path: WindowsPath) -> Self {
        Self {
            id,
            name: name.into(),
            path,
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
        }
    }

    pub fn update_name(&mut self, name: impl Into<String>) -> Result<(), String> {
        let new_name = name.into();
        if new_name.is_empty() {
            return Err(format!("Folder name cannot be empty"));
        }
        self.name = new_name;
        self.updated_at = SystemTime::now();
        Ok(())
    }

    pub fn update_path(&mut self, new_path: WindowsPath) -> Result<(), DomainError> {
        if new_path.is_root() {
            return Err(DomainError::IllegalDirectoryTarget);
        }
        self.path = new_path;
        self.updated_at = SystemTime::now();
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
        let f = Folder::new("Docs", test_path("C:\\Docs"));
        assert_eq!(f.name, "Docs");
    }

    #[test]
    fn test_update_name() {
        let mut f = Folder::new("Docs", test_path("C:\\Docs"));
        f.update_name("Projects").unwrap();
        assert_eq!(f.name, "Projects");
    }
}