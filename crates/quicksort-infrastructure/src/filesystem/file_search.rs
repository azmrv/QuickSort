//! Standard file search implementation.
//!
//! Walks the file system recursively from configured directories,
//! applies filters from SearchQuery, and returns matching results.

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use tokio::fs;

use quicksort_application::errors::UseCaseError;
use quicksort_application::ports::outbound::{FileSearchPort, FileSearchResult, SearchResult};
use quicksort_domain::{DateFilter, SearchFilter, SearchQuery};

/// Real file system search implementation.
pub struct FsFileSearch;

impl FsFileSearch {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FsFileSearch {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FileSearchPort for FsFileSearch {
    async fn search(
        &self,
        directories: &[String],
        query_text: &str,
        max_results: usize,
    ) -> Result<SearchResult, UseCaseError> {
        let start = std::time::Instant::now();

        let query = SearchQuery::parse(query_text).map_err(|e| {
            UseCaseError::InvalidCommand(format!("Invalid search query: {}", e))
        })?;

        let mut results = Vec::new();

        for dir_str in directories {
            let dir = Path::new(dir_str);
            if !dir.exists() || !dir.is_dir() {
                continue;
            }
            self.walk_directory(dir, &query, &mut results, max_results)
                .await;
        }

        let truncated = results.len() >= max_results;
        let elapsed = start.elapsed().as_millis() as u64;

        Ok(SearchResult {
            total_count: results.len(),
            files: results,
            search_time_ms: elapsed,
            truncated,
        })
    }
}

impl FsFileSearch {
    /// Recursively walk a directory, collecting matching entries.
    fn walk_directory<'a>(
        &'a self,
        dir: &'a Path,
        query: &'a SearchQuery,
        results: &'a mut Vec<FileSearchResult>,
        max_results: usize,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            if results.len() >= max_results {
                return;
            }

            let entries = match fs::read_dir(dir).await {
                Ok(entries) => entries,
                Err(_) => return,
            };

            let mut entries = entries;
            while let Some(entry) = entries.next_entry().await.unwrap_or(None) {
                if results.len() >= max_results {
                    return;
                }

                let path = entry.path();
                let metadata = match fs::metadata(&path).await {
                    Ok(m) => m,
                    Err(_) => continue,
                };

                let name = match path.file_name() {
                    Some(n) => n.to_string_lossy().to_string(),
                    None => continue,
                };

                let is_directory = metadata.is_dir();
                let size = metadata.len();

                let modified_at = metadata
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                    .map(|d| d.as_secs() as i64);

                if self.matches_query(&name, is_directory, size, modified_at, query) {
                    results.push(FileSearchResult {
                        path: path.to_string_lossy().to_string(),
                        name,
                        size,
                        is_directory,
                        modified_at,
                    });
                }

                if is_directory && results.len() < max_results {
                    self.walk_directory(&path, query, results, max_results).await;
                }
            }
        })
    }

    /// Check if a file/directory matches the parsed search query.
    fn matches_query(
        &self,
        name: &str,
        is_dir: bool,
        size: u64,
        modified_at: Option<i64>,
        query: &SearchQuery,
    ) -> bool {
        // Check if query is empty — match everything
        if query.is_empty() {
            return true;
        }

        // Check filters first (fast)
        for filter in &query.filters {
            match filter {
                SearchFilter::Extension(ext) => {
                    let file_ext = name
                        .rsplit('.')
                        .next()
                        .unwrap_or("")
                        .to_lowercase();
                    if file_ext != *ext {
                        return false;
                    }
                }
                SearchFilter::Size(op, target) => {
                    if !op.compare(size, *target) {
                        return false;
                    }
                }
                SearchFilter::DateModified(date_filter) => {
                    let Some(modified) = modified_at else {
                        return false;
                    };
                    if !self.matches_date(modified, date_filter) {
                        return false;
                    }
                }
                SearchFilter::FoldersOnly => {
                    if !is_dir {
                        return false;
                    }
                }
                SearchFilter::FilesOnly => {
                    if is_dir {
                        return false;
                    }
                }
            }
        }

        // Check text terms (ALL must match)
        let name_lower = name.to_lowercase();
        for term in &query.text_terms {
            if !self.matches_term(&name_lower, term) {
                return false;
            }
        }

        // Check excluded terms (NONE must match)
        for term in &query.excluded_terms {
            if self.matches_term(&name_lower, term) {
                return false;
            }
        }

        // Check OR groups (at least one group must have a matching term)
        if !query.or_groups.is_empty() {
            let any_group_matches = query.or_groups.iter().any(|group| {
                group.iter().any(|term| self.matches_term(&name_lower, term))
            });
            if !any_group_matches {
                return false;
            }
        }

        true
    }

    /// Check if a name matches a term (supports * and ? wildcards).
    fn matches_term(&self, name_lower: &str, term: &str) -> bool {
        let term_lower = term.to_lowercase();

        if term_lower.contains('*') || term_lower.contains('?') {
            self.wildcard_match(name_lower, &term_lower)
        } else {
            name_lower.contains(term_lower.as_str())
        }
    }

    /// Simple wildcard matching (* = any chars, ? = single char).
    fn wildcard_match(&self, text: &str, pattern: &str) -> bool {
        let text: Vec<char> = text.chars().collect();
        let pattern: Vec<char> = pattern.chars().collect();
        self.wildcard_match_recursive(&text, &pattern, 0, 0)
    }

    fn wildcard_match_recursive(&self, text: &[char], pattern: &[char], ti: usize, pi: usize) -> bool {
        if pi == pattern.len() {
            return ti == text.len();
        }

        if pattern[pi] == '*' {
            // Try matching zero or more characters
            for skip in 0..=text.len() - ti {
                if self.wildcard_match_recursive(text, pattern, ti + skip, pi + 1) {
                    return true;
                }
            }
            false
        } else if ti < text.len() && (pattern[pi] == '?' || pattern[pi] == text[ti]) {
            self.wildcard_match_recursive(text, pattern, ti + 1, pi + 1)
        } else {
            false
        }
    }

    /// Check if a modification timestamp matches a date filter.
    fn matches_date(&self, modified_secs: i64, filter: &DateFilter) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let day_secs = 86400;
        let start_of_today = now - (now % day_secs);

        match filter {
            DateFilter::Today => {
                modified_secs >= start_of_today
            }
            DateFilter::Yesterday => {
                modified_secs >= start_of_today - day_secs
                    && modified_secs < start_of_today
            }
            DateFilter::PastDays(days) => {
                let cutoff = start_of_today - (*days as i64) * day_secs;
                modified_secs >= cutoff
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_search_empty_query_matches_all() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("a.txt"), "hello").await.unwrap();
        fs::write(dir.path().join("b.pdf"), "world").await.unwrap();

        let search = FsFileSearch::new();
        let dirs = vec![dir.path().to_str().unwrap().to_string()];
        let result = search.search(&dirs, "", 100).await.unwrap();

        assert_eq!(result.files.len(), 2);
    }

    #[tokio::test]
    async fn test_search_ext_filter() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("doc.pdf"), "pdf").await.unwrap();
        fs::write(dir.path().join("doc.txt"), "txt").await.unwrap();

        let search = FsFileSearch::new();
        let dirs = vec![dir.path().to_str().unwrap().to_string()];
        let result = search.search(&dirs, "ext:pdf", 100).await.unwrap();

        assert_eq!(result.files.len(), 1);
        assert_eq!(result.files[0].name, "doc.pdf");
    }

    #[tokio::test]
    async fn test_search_text_term() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("report_v1.pdf"), "").await.unwrap();
        fs::write(dir.path().join("summary.pdf"), "").await.unwrap();

        let search = FsFileSearch::new();
        let dirs = vec![dir.path().to_str().unwrap().to_string()];
        let result = search.search(&dirs, "report", 100).await.unwrap();

        assert_eq!(result.files.len(), 1);
        assert_eq!(result.files[0].name, "report_v1.pdf");
    }

    #[tokio::test]
    async fn test_search_size_gt() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("big.bin"), vec![0u8; 2000]).await.unwrap();
        fs::write(dir.path().join("small.bin"), vec![0u8; 100]).await.unwrap();

        let search = FsFileSearch::new();
        let dirs = vec![dir.path().to_str().unwrap().to_string()];
        let result = search.search(&dirs, "size:>1kb", 100).await.unwrap();

        assert_eq!(result.files.len(), 1);
        assert_eq!(result.files[0].name, "big.bin");
    }

    #[tokio::test]
    async fn test_search_folders_only() {
        let dir = tempdir().unwrap();
        fs::create_dir(dir.path().join("subdir")).await.unwrap();
        fs::write(dir.path().join("file.txt"), "").await.unwrap();

        let search = FsFileSearch::new();
        let dirs = vec![dir.path().to_str().unwrap().to_string()];
        let result = search.search(&dirs, "folders:", 100).await.unwrap();

        assert_eq!(result.files.len(), 1);
        assert!(result.files[0].is_directory);
    }

    #[tokio::test]
    async fn test_search_max_results() {
        let dir = tempdir().unwrap();
        for i in 0..10 {
            fs::write(dir.path().join(format!("file{}.txt", i)), "").await.unwrap();
        }

        let search = FsFileSearch::new();
        let dirs = vec![dir.path().to_str().unwrap().to_string()];
        let result = search.search(&dirs, "ext:txt", 3).await.unwrap();

        assert_eq!(result.files.len(), 3);
        assert!(result.truncated);
    }

    #[tokio::test]
    async fn test_search_result_metadata() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("data.bin"), vec![0u8; 42]).await.unwrap();

        let search = FsFileSearch::new();
        let dirs = vec![dir.path().to_str().unwrap().to_string()];
        let result = search.search(&dirs, "ext:bin", 100).await.unwrap();

        assert_eq!(result.files.len(), 1);
        assert_eq!(result.files[0].size, 42);
        assert!(!result.files[0].is_directory);
        assert!(result.files[0].modified_at.is_some());
        assert!(result.search_time_ms < 5000);
    }

    #[tokio::test]
    async fn test_search_excluded_term() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("report.pdf"), "").await.unwrap();
        fs::write(dir.path().join("draft.pdf"), "").await.unwrap();

        let search = FsFileSearch::new();
        let dirs = vec![dir.path().to_str().unwrap().to_string()];
        let result = search.search(&dirs, "ext:pdf !draft", 100).await.unwrap();

        assert_eq!(result.files.len(), 1);
        assert_eq!(result.files[0].name, "report.pdf");
    }

    #[tokio::test]
    async fn test_search_nonexistent_directory() {
        let search = FsFileSearch::new();
        let dirs = vec!["C:\\nonexistent_directory_xyz".to_string()];
        let result = search.search(&dirs, "ext:txt", 100).await.unwrap();

        assert_eq!(result.files.len(), 0);
    }

    #[tokio::test]
    async fn test_search_wildcard() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("report_v1.pdf"), "").await.unwrap();
        fs::write(dir.path().join("report_v2.pdf"), "").await.unwrap();
        fs::write(dir.path().join("summary.pdf"), "").await.unwrap();

        let search = FsFileSearch::new();
        let dirs = vec![dir.path().to_str().unwrap().to_string()];
        let result = search.search(&dirs, "report*", 100).await.unwrap();

        assert_eq!(result.files.len(), 2);
    }
}
