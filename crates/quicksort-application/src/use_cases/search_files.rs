//! SearchFiles Use Case
//!
//! Parses a search query string and executes it against the file system
//! via the FileSearchPort. Returns matching files with metadata.

use std::sync::Arc;
use std::time::Instant;

use async_trait::async_trait;
use tracing::instrument;

use crate::errors::UseCaseError;
use crate::ports::outbound::{FileSearchPort, SearchResult};
use quicksort_domain::SearchQuery;

/// Inbound port for file search operations.
#[async_trait]
pub trait SearchFiles: Send + Sync {
    /// Search for files matching the query string.
    ///
    /// # Arguments
    /// * `query_text` — raw search query (e.g., "ext:pdf size:>10mb")
    /// * `directories` — directories to search in
    ///
    /// # Returns
    /// `SearchResult` with matching files, or `UseCaseError` on parse failure.
    async fn search(
        &self,
        query_text: &str,
        directories: &[String],
    ) -> Result<SearchResult, UseCaseError>;
}

/// Default max results for search.
const DEFAULT_MAX_RESULTS: usize = 200;

/// Concrete implementation of SearchFiles use case.
pub struct SearchFilesUseCase {
    file_search: Arc<dyn FileSearchPort>,
}

impl SearchFilesUseCase {
    pub fn new(file_search: Arc<dyn FileSearchPort>) -> Self {
        Self { file_search }
    }
}

#[async_trait]
impl SearchFiles for SearchFilesUseCase {
    #[instrument(
        name = "search_files",
        skip_all,
        fields(
            query_text = %query_text,
            directory_count = %directories.len(),
            result_count,
            duration_ms
        )
    )]
    async fn search(
        &self,
        query_text: &str,
        directories: &[String],
    ) -> Result<SearchResult, UseCaseError> {
        let started_at = Instant::now();

        // Parse the query
        let _query = SearchQuery::parse(query_text)
            .map_err(|e| UseCaseError::InvalidCommand(format!("Invalid search query: {}", e)))?;

        // Delegate to the file search port
        let result = self
            .file_search
            .search(directories, query_text, DEFAULT_MAX_RESULTS)
            .await?;

        let span = tracing::Span::current();
        span.record("result_count", result.total_count);
        span.record("duration_ms", started_at.elapsed().as_millis() as u64);

        tracing::info!(
            query_text = %query_text,
            directory_count = %directories.len(),
            total_count = result.total_count,
            search_time_ms = result.search_time_ms,
            truncated = result.truncated,
            "search completed"
        );

        Ok(result)
    }
}
