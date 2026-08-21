//! Quick duplicate check based on file name.

use async_trait::async_trait;
use quicksort_domain::{
    DomainError, DuplicateCheckMode, DuplicateCheckResult, DuplicateChecker, WindowsPath,
};

/// Checks for duplicates by file name only.
pub struct NameChecker;

#[async_trait]
impl DuplicateChecker for NameChecker {
    async fn check(
        &self,
        source: &WindowsPath,
        destination: &WindowsPath,
        mode: &DuplicateCheckMode,
    ) -> Result<DuplicateCheckResult, DomainError> {
        // Only check if mode is Name
        if *mode != DuplicateCheckMode::Name {
            return Ok(DuplicateCheckResult {
                source: source.clone(),
                destination: destination.clone(),
                exists: false,
                mode: mode.clone(),
            });
        }

        let exists = tokio::fs::metadata(destination.to_path_buf()).await.is_ok();

        Ok(DuplicateCheckResult {
            source: source.clone(),
            destination: destination.clone(),
            exists,
            mode: mode.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_no_duplicate_when_file_not_exists() {
        let dir = tempdir().unwrap();
        let source = WindowsPath::new(dir.path().join("source.txt").to_str().unwrap()).unwrap();
        let dest = WindowsPath::new(dir.path().join("dest.txt").to_str().unwrap()).unwrap();

        let checker = NameChecker;
        let result = checker
            .check(&source, &dest, &DuplicateCheckMode::Name)
            .await
            .unwrap();

        assert!(!result.exists);
    }

    #[tokio::test]
    async fn test_duplicate_when_file_exists() {
        let dir = tempdir().unwrap();
        let source = WindowsPath::new(dir.path().join("source.txt").to_str().unwrap()).unwrap();
        let dest = WindowsPath::new(dir.path().join("dest.txt").to_str().unwrap()).unwrap();

        // Create the destination file
        tokio::fs::write(dest.to_path_buf(), "content")
            .await
            .unwrap();

        let checker = NameChecker;
        let result = checker
            .check(&source, &dest, &DuplicateCheckMode::Name)
            .await
            .unwrap();

        assert!(result.exists);
    }

    #[tokio::test]
    async fn test_skips_when_mode_is_not_name() {
        let dir = tempdir().unwrap();
        let source = WindowsPath::new(dir.path().join("source.txt").to_str().unwrap()).unwrap();
        let dest = WindowsPath::new(dir.path().join("dest.txt").to_str().unwrap()).unwrap();

        // Create the destination file
        tokio::fs::write(dest.to_path_buf(), "content")
            .await
            .unwrap();

        let checker = NameChecker;
        let result = checker
            .check(&source, &dest, &DuplicateCheckMode::Size)
            .await
            .unwrap();

        assert!(!result.exists);
    }
}
