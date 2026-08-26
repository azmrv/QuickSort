//! Medium duplicate check based on file name and size.

use async_trait::async_trait;
use quicksort_domain::{
    AbsolutePath, DomainError, DuplicateCheckMode, DuplicateCheckResult, DuplicateChecker,
};

/// Checks for duplicates by file name and size.
pub struct SizeChecker;

#[async_trait]
impl DuplicateChecker for SizeChecker {
    async fn check(
        &self,
        source: &AbsolutePath,
        destination: &AbsolutePath,
        mode: &DuplicateCheckMode,
    ) -> Result<DuplicateCheckResult, DomainError> {
        // Only check if mode is Size
        if *mode != DuplicateCheckMode::Size {
            return Ok(DuplicateCheckResult {
                source: source.clone(),
                destination: destination.clone(),
                exists: false,
                mode: mode.clone(),
            });
        }

        // Check if destination exists
        let dest_metadata = match tokio::fs::metadata(destination.to_path_buf()).await {
            Ok(m) => m,
            Err(_) => {
                return Ok(DuplicateCheckResult {
                    source: source.clone(),
                    destination: destination.clone(),
                    exists: false,
                    mode: mode.clone(),
                });
            }
        };

        // Get source metadata
        let source_metadata = match tokio::fs::metadata(source.to_path_buf()).await {
            Ok(m) => m,
            Err(_) => {
                return Ok(DuplicateCheckResult {
                    source: source.clone(),
                    destination: destination.clone(),
                    exists: false,
                    mode: mode.clone(),
                });
            }
        };

        // Compare sizes
        let exists = source_metadata.len() == dest_metadata.len();

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
        let source = AbsolutePath::new(dir.path().join("source.txt").to_str().unwrap()).unwrap();
        let dest = AbsolutePath::new(dir.path().join("dest.txt").to_str().unwrap()).unwrap();

        let checker = SizeChecker;
        let result = checker
            .check(&source, &dest, &DuplicateCheckMode::Size)
            .await
            .unwrap();

        assert!(!result.exists);
    }

    #[tokio::test]
    async fn test_duplicate_when_same_size() {
        let dir = tempdir().unwrap();
        let source = AbsolutePath::new(dir.path().join("source.txt").to_str().unwrap()).unwrap();
        let dest = AbsolutePath::new(dir.path().join("dest.txt").to_str().unwrap()).unwrap();

        // Create both files with same content
        tokio::fs::write(source.to_path_buf(), "hello")
            .await
            .unwrap();
        tokio::fs::write(dest.to_path_buf(), "world").await.unwrap();

        let checker = SizeChecker;
        let result = checker
            .check(&source, &dest, &DuplicateCheckMode::Size)
            .await
            .unwrap();

        assert!(result.exists);
    }

    #[tokio::test]
    async fn test_no_duplicate_when_different_size() {
        let dir = tempdir().unwrap();
        let source = AbsolutePath::new(dir.path().join("source.txt").to_str().unwrap()).unwrap();
        let dest = AbsolutePath::new(dir.path().join("dest.txt").to_str().unwrap()).unwrap();

        // Create files with different content
        tokio::fs::write(source.to_path_buf(), "hello")
            .await
            .unwrap();
        tokio::fs::write(dest.to_path_buf(), "world!")
            .await
            .unwrap();

        let checker = SizeChecker;
        let result = checker
            .check(&source, &dest, &DuplicateCheckMode::Size)
            .await
            .unwrap();

        assert!(!result.exists);
    }
}
