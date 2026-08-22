//! File search port — performs file system search queries.
//!
//! This port is implemented by infrastructure (FsFileSearch) and used
//! by the SearchFiles use case to execute file searches.

use async_trait::async_trait;

use crate::errors::UseCaseError;

/// A single file search result.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileSearchResult {
    /// Full path to the file.
    pub path: String,
    /// File name (last component of path).
    pub name: String,
    /// File size in bytes.
    pub size: u64,
    /// Whether this is a directory.
    pub is_directory: bool,
    /// Last modification time (Unix timestamp).
    pub modified_at: Option<i64>,
}

/// Search results with metadata.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SearchResult {
    /// Matching files/directories.
    pub files: Vec<FileSearchResult>,
    /// Total number of results found.
    pub total_count: usize,
    /// Search duration in milliseconds.
    pub search_time_ms: u64,
    /// Whether results were truncated by the limit.
    pub truncated: bool,
}

/// Outbound port for file system search.
///
/// Implemented by `FsFileSearch` in the Infrastructure layer.
#[async_trait]
pub trait FileSearchPort: Send + Sync {
    /// Search for files matching the given query.
    ///
    /// # Arguments
    /// * `directories` — directories to search in (from configured folders)
    /// * `query_text` — raw search query text (parsed by use case)
    /// * `max_results` — maximum number of results to return
    ///
    /// # Returns
    /// `SearchResult` with matching files.
    async fn search(
        &self,
        directories: &[String],
        query_text: &str,
        max_results: usize,
    ) -> Result<SearchResult, UseCaseError>;
}
