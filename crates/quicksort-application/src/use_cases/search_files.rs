//! SearchFiles Use Case
//!
//! Parses a search query string and executes it against the file system
//! via the FileSearchPort. Returns matching files with metadata.

use std::sync::Arc;

use async_trait::async_trait;

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
    async fn search(
        &self,
        query_text: &str,
        directories: &[String],
    ) -> Result<SearchResult, UseCaseError> {
        // Parse the query
        let _query = SearchQuery::parse(query_text).map_err(|e| {
            UseCaseError::InvalidCommand(format!("Invalid search query: {}", e))
        })?;

        // Delegate to the file search port
        self.file_search
            .search(directories, query_text, DEFAULT_MAX_RESULTS)
            .await
    }
}
