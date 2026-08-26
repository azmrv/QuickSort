//! Domain entities for duplicate file detection.

use crate::value_objects::AbsolutePath;
use serde::{Deserialize, Serialize};

/// Result of a duplicate check for a single file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DuplicateCheckResult {
    /// Source file path.
    pub source: AbsolutePath,
    /// Destination file path.
    pub destination: AbsolutePath,
    /// Whether a duplicate was found.
    pub exists: bool,
    /// Mode used for the check.
    pub mode: DuplicateCheckMode,
}

/// Duplicate detection mode.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DuplicateCheckMode {
    /// Quick check: file with same name exists at destination.
    #[default]
    Name,
    /// Medium check: same name AND same file size.
    Size,
    /// Deep check: SHA-256 hash comparison (slowest, most accurate).
    Content,
}

/// Service for checking duplicate files.
#[async_trait::async_trait]
pub trait DuplicateChecker: Send + Sync {
    /// Check if a file already exists at the destination.
    async fn check(
        &self,
        source: &AbsolutePath,
        destination: &AbsolutePath,
        mode: &DuplicateCheckMode,
    ) -> Result<DuplicateCheckResult, crate::errors::DomainError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_mode() {
        assert_eq!(DuplicateCheckMode::default(), DuplicateCheckMode::Name);
    }

    #[test]
    fn test_serialize_modes() {
        let name = DuplicateCheckMode::Name;
        let json = serde_json::to_string(&name).unwrap();
        assert_eq!(json, "\"name\"");

        let size = DuplicateCheckMode::Size;
        let json = serde_json::to_string(&size).unwrap();
        assert_eq!(json, "\"size\"");

        let content = DuplicateCheckMode::Content;
        let json = serde_json::to_string(&content).unwrap();
        assert_eq!(json, "\"content\"");
    }
}
