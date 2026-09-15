//! Standard file search implementation.
//!
//! Walks the file system recursively from configured directories,
//! applies filters from SearchQuery, and returns matching results.

use std::path::{Path, PathBuf};
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

        let query = SearchQuery::parse(query_text)
            .map_err(|e| UseCaseError::InvalidCommand(format!("Invalid search query: {}", e)))?;

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
    /// Max tree depth the walk will descend into. A real chain deeper than this
    /// implies a cycle that the symlink check cannot see (nested real dirs), or
    /// software-generated runaway nesting. Either way, stop.
    const MAX_DEPTH: u32 = 512;

    /// Walk a directory tree iteratively, collecting matching entries.
    ///
    /// Uses an explicit heap stack instead of recursion: a recursive
    /// `Box::pin(async move { ... .await })` chain keeps one poll frame per
    /// level on the thread stack and overflows 1 MiB at ~500 nested directories
    /// (QA 2026-09-11, 0xc00000fd on a Desktop chain of 541 real folders).
    /// The future state machine here is O(1) in depth — only leaf ops are awaited.
    async fn walk_directory(
        &self,
        start_dir: &Path,
        query: &SearchQuery,
        results: &mut Vec<FileSearchResult>,
        max_results: usize,
    ) {
        // (path, depth) pairs; depth of a child = parent depth + 1
        let mut stack: Vec<(PathBuf, u32)> = vec![(start_dir.to_path_buf(), 0)];

        while let Some((dir, depth)) = stack.pop() {
            if results.len() >= max_results {
                return;
            }

            if depth > Self::MAX_DEPTH {
                tracing::warn!(
                    "search: tree deeper than {} levels at {:?}; stopping descent \
                     (possible cycle or runaway directory chain)",
                    Self::MAX_DEPTH,
                    dir
                );
                continue;
            }

            let mut entries = match fs::read_dir(&dir).await {
                Ok(entries) => entries,
                Err(e) => {
                    tracing::warn!("search: cannot read directory {:?}: {}", dir, e);
                    continue;
                }
            };

            while let Some(entry) = entries.next_entry().await.unwrap_or(None) {
                if results.len() >= max_results {
                    return;
                }

                let path = entry.path();
                // Do not follow symlinks/junctions while walking: resolving a
                // reparse point can escape the scanned tree (e.g. Windows
                // `AppData\Local\Application Data` → `AppData\Local`) and form
                // a cycle, which previously caused unbounded recursion and a
                // silent app crash (QA 2026-09-11, search "123" over AppData).
                // DirEntry::file_type reads the entry itself and does NOT
                // resolve the reparse target, so junctions are seen as symlinks.
                let file_type = match entry.file_type().await {
                    Ok(ft) => ft,
                    Err(e) => {
                        tracing::debug!("search: cannot stat {:?}: {}", path, e);
                        continue;
                    }
                };
                if file_type.is_symlink() {
                    tracing::debug!("search: skipping symlink/junction {:?}", path);
                    continue;
                }

                let metadata = match fs::metadata(&path).await {
                    Ok(m) => m,
                    Err(e) => {
                        tracing::debug!("search: cannot stat {:?}: {}", path, e);
                        continue;
                    }
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
                    stack.push((path, depth + 1));
                }
            }
        }
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
                    let file_ext = name.rsplit('.').next().unwrap_or("").to_lowercase();
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
                group
                    .iter()
                    .any(|term| self.matches_term(&name_lower, term))
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

    fn wildcard_match_recursive(
        &self,
        text: &[char],
        pattern: &[char],
        ti: usize,
        pi: usize,
    ) -> bool {
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
            DateFilter::Today => modified_secs >= start_of_today,
            DateFilter::Yesterday => {
                modified_secs >= start_of_today - day_secs && modified_secs < start_of_today
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
        fs::write(dir.path().join("report_v1.pdf"), "")
            .await
            .unwrap();
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
        fs::write(dir.path().join("big.bin"), vec![0u8; 2000])
            .await
            .unwrap();
        fs::write(dir.path().join("small.bin"), vec![0u8; 100])
            .await
            .unwrap();

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
            fs::write(dir.path().join(format!("file{}.txt", i)), "")
                .await
                .unwrap();
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
        fs::write(dir.path().join("data.bin"), vec![0u8; 42])
            .await
            .unwrap();

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
        fs::write(dir.path().join("report_v1.pdf"), "")
            .await
            .unwrap();
        fs::write(dir.path().join("report_v2.pdf"), "")
            .await
            .unwrap();
        fs::write(dir.path().join("summary.pdf"), "").await.unwrap();

        let search = FsFileSearch::new();
        let dirs = vec![dir.path().to_str().unwrap().to_string()];
        let result = search.search(&dirs, "report*", 100).await.unwrap();

        assert_eq!(result.files.len(), 2);
    }

    /// Regression: walking must skip Windows junctions, which can form cycles
    /// (e.g. `AppData\Local\Application Data` → `AppData\Local`) and previously
    /// caused unbounded recursion and a silent app crash (QA 2026-09-11,
    /// search "123" over the whole AppData folder).
    #[cfg(windows)]
    #[tokio::test]
    async fn test_search_skips_junction_cycle() {
        use std::process::Command;

        let base = tempdir().unwrap();
        let base_path = base.path();

        fs::write(base_path.join("a123.txt"), "").await.unwrap();
        fs::create_dir(base_path.join("sub")).await.unwrap();
        fs::write(base_path.join("sub/b123.txt"), "").await.unwrap();

        // `mklink /J` creates a directory junction without admin rights
        // (unlike symlinks). Point it at the base dir itself → a cycle.
        let link = base_path.join("loop");
        let status = Command::new("cmd")
            .args(["/C", "mklink", "/J"])
            .arg(link)
            .arg(base_path)
            .status()
            .expect("failed to run mklink");
        assert!(status.success(), "mklink /J failed: {status:?}");

        let search = FsFileSearch::new();
        let dirs = vec![base_path.to_str().unwrap().to_string()];

        let result = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            search.search(&dirs, "123", 10_000),
        )
        .await
        .expect("search did not finish in 5s (junction recursion?)")
        .unwrap();

        let mut names: Vec<&str> = result.files.iter().map(|f| f.name.as_str()).collect();
        names.sort_unstable();
        assert_eq!(names, ["a123.txt", "b123.txt"]);
    }

    /// Regression: a recursive walk overflows the thread stack on deep *real*
    /// directory chains (no symlinks involved) — QA 2026-09-11, 0xc00000fd on a
    /// Desktop chain of 541 nested folders. The iterative walk must survive
    /// deeper chains and enforce MAX_DEPTH.
    #[tokio::test]
    async fn test_search_deep_chain_no_overflow() {
        let base = tempdir().unwrap();
        // Windows caps non-prefixed paths at 260 chars (MAX_PATH); a chain deep
        // enough to overflow recursion needs the long-path `\\?\` prefix.
        let root: PathBuf = if cfg!(windows) {
            PathBuf::from(format!(r"\\?\{}", base.path().display()))
        } else {
            base.path().to_path_buf()
        };

        // Chain deeper than MAX_DEPTH so the guard is exercised, but derived from
        // the constant so the fixture stays valid if MAX_DEPTH is bumped.
        let levels = FsFileSearch::MAX_DEPTH as usize + 100;
        let mut cur = root.clone();
        for i in 0..levels {
            cur = cur.join(format!("d{i}"));
            fs::create_dir(&cur).await.unwrap();
        }

        let search = FsFileSearch::new();
        let dirs = vec![root.to_str().unwrap().to_string()];

        let result = tokio::time::timeout(
            std::time::Duration::from_secs(10),
            search.search(&dirs, "", 10_000),
        )
        .await
        .expect("search did not finish in 10s (deep chain overflow?)")
        .unwrap();

        // Empty query matches every directory. Each dir is matched while its
        // *parent* is being read (root matches d0, then d0..d511 at depths
        // 1..=512 each match their single child) → 1 + MAX_DEPTH results total.
        // d512 itself is never read: it is only pushed at depth 513 and killed
        // by the guard on pop, before its entries are enumerated.
        assert_eq!(result.files.len(), FsFileSearch::MAX_DEPTH as usize + 1);
        assert!(!result.truncated);

        // Long-path-safe cleanup; tempdir tolerates the now-missing dir.
        let _ = std::fs::remove_dir_all(&root);
    }
}
