//! Value objects – immutable, self-validating types.

use crate::errors::DomainError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowsPath(String);

impl WindowsPath {
    pub fn new(path: &str) -> Result<Self, DomainError> {
        let sanitized = path.replace('/', "\\");
        if sanitized.is_empty() {
            return Err(DomainError::EmptyPath);
        }
        // Minimal validation for now (will be expanded later)
        Ok(Self(sanitized))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn is_root(&self) -> bool {
        self.0.len() == 3 && self.0.ends_with(":\\")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FolderId(pub String);

impl FolderId {
    pub fn from_string(s: impl Into<String>) -> Self {
        Self(s.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OperationId(pub String);

impl OperationId {
    pub fn from_string(s: impl Into<String>) -> Self {
        Self(s.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}