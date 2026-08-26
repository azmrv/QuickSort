//! Infrastructure adapter for DuplicateDetectionPort.

use async_trait::async_trait;
use quicksort_application::ports::outbound::DuplicateDetectionPort;
use quicksort_domain::{
    AbsolutePath, DomainError, DuplicateCheckMode, DuplicateCheckResult, DuplicateChecker,
};

/// Adapter that delegates DuplicateDetectionPort to a concrete DuplicateChecker.
pub struct DuplicateDetectionAdapter<C: DuplicateChecker> {
    checker: C,
}

impl<C: DuplicateChecker> DuplicateDetectionAdapter<C> {
    pub fn new(checker: C) -> Self {
        Self { checker }
    }
}

#[async_trait]
impl<C: DuplicateChecker + Send + Sync> DuplicateDetectionPort for DuplicateDetectionAdapter<C> {
    async fn check_duplicate(
        &self,
        source: &AbsolutePath,
        destination: &AbsolutePath,
        mode: &DuplicateCheckMode,
    ) -> Result<DuplicateCheckResult, DomainError> {
        self.checker.check(source, destination, mode).await
    }
}
