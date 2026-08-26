//! Deep duplicate check based on SHA-256 content hash.

use async_trait::async_trait;
use quicksort_domain::{
    AbsolutePath, DomainError, DuplicateCheckMode, DuplicateCheckResult, DuplicateChecker,
};
use sha2::{Digest, Sha256};
use tokio::fs::File;
use tokio::io::AsyncReadExt;

/// Checks for duplicates by SHA-256 content hash.
pub struct ContentChecker;

impl ContentChecker {
    /// Calculate SHA-256 hash of a file.
    async fn hash_file(path: &AbsolutePath) -> Result<Vec<u8>, DomainError> {
        let mut file = File::open(path.to_path_buf())
            .await
            .map_err(|e| DomainError::InvalidPath(e.to_string()))?;

        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 8192];

        loop {
            let bytes_read = file
                .read(&mut buffer)
                .await
                .map_err(|e| DomainError::InvalidPath(e.to_string()))?;

            if bytes_read == 0 {
                break;
            }

            hasher.update(&buffer[..bytes_read]);
        }

        Ok(hasher.finalize().to_vec())
    }
}

#[async_trait]
impl DuplicateChecker for ContentChecker {
    async fn check(
        &self,
        source: &AbsolutePath,
        destination: &AbsolutePath,
        mode: &DuplicateCheckMode,
    ) -> Result<DuplicateCheckResult, DomainError> {
        // Only check if mode is Content
        if *mode != DuplicateCheckMode::Content {
            return Ok(DuplicateCheckResult {
                source: source.clone(),
                destination: destination.clone(),
                exists: false,
                mode: mode.clone(),
            });
        }

        // Check if both files exist
        let source_exists = tokio::fs::metadata(source.to_path_buf()).await.is_ok();
        let dest_exists = tokio::fs::metadata(destination.to_path_buf()).await.is_ok();

        if !source_exists || !dest_exists {
            return Ok(DuplicateCheckResult {
                source: source.clone(),
                destination: destination.clone(),
                exists: false,
                mode: mode.clone(),
            });
        }

        // Calculate hashes
        let source_hash = Self::hash_file(source).await?;
        let dest_hash = Self::hash_file(destination).await?;

        let exists = source_hash == dest_hash;

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

        let checker = ContentChecker;
        let result = checker
            .check(&source, &dest, &DuplicateCheckMode::Content)
            .await
            .unwrap();

        assert!(!result.exists);
    }

    #[tokio::test]
    async fn test_duplicate_when_same_content() {
        let dir = tempdir().unwrap();
        let source = AbsolutePath::new(dir.path().join("source.txt").to_str().unwrap()).unwrap();
        let dest = AbsolutePath::new(dir.path().join("dest.txt").to_str().unwrap()).unwrap();

        // Create both files with same content
        tokio::fs::write(source.to_path_buf(), "hello world")
            .await
            .unwrap();
        tokio::fs::write(dest.to_path_buf(), "hello world")
            .await
            .unwrap();

        let checker = ContentChecker;
        let result = checker
            .check(&source, &dest, &DuplicateCheckMode::Content)
            .await
            .unwrap();

        assert!(result.exists);
    }

    #[tokio::test]
    async fn test_no_duplicate_when_different_content() {
        let dir = tempdir().unwrap();
        let source = AbsolutePath::new(dir.path().join("source.txt").to_str().unwrap()).unwrap();
        let dest = AbsolutePath::new(dir.path().join("dest.txt").to_str().unwrap()).unwrap();

        // Create files with different content
        tokio::fs::write(source.to_path_buf(), "hello")
            .await
            .unwrap();
        tokio::fs::write(dest.to_path_buf(), "world").await.unwrap();

        let checker = ContentChecker;
        let result = checker
            .check(&source, &dest, &DuplicateCheckMode::Content)
            .await
            .unwrap();

        assert!(!result.exists);
    }
}
