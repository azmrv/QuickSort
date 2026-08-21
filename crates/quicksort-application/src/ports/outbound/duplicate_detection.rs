//! Outbound port for duplicate file detection.

use async_trait::async_trait;
use quicksort_domain::{DuplicateCheckMode, DuplicateCheckResult};

/// Port for checking duplicate files at the destination.
///
/// Infrastructure provides the concrete implementation (NameChecker,
/// SizeChecker, ContentChecker) selected based on the active mode.
#[async_trait]
pub trait DuplicateDetectionPort: Send + Sync {
    /// Check if a file already exists at the destination.
    async fn check_duplicate(
        &self,
        source: &quicksort_domain::value_objects::WindowsPath,
        destination: &quicksort_domain::value_objects::WindowsPath,
        mode: &DuplicateCheckMode,
    ) -> Result<DuplicateCheckResult, quicksort_domain::errors::DomainError>;
}
