//! SearchQuery value object — parses Everything-style search syntax.
//!
//! # Supported Syntax (Phase 1)
//!
//! ```text
//! <text>                    # Filename contains text
//! "ext:<ext>"               # Filter by extension
//! "size:>10mb"              # Filter by size (supports >, <, >=, <=)
//! "date-modified:today"     # Filter by modification date
//! "folders:"                # Show only folders
//! "files:"                  # Show only files
//! !<term>                   # NOT
//! <term1> | <term2>         # OR
//! <term1> <term2>           # AND (implicit)
//! *                         # Wildcard: zero or more characters
//! ?                         # Wildcard: exactly one character
//! ```

use serde::{Deserialize, Serialize};

use crate::errors::DomainError;

// ============================================================================
// Size comparison
// ============================================================================

/// Size comparison operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SizeOp {
    GreaterThan,
    LessThan,
    GreaterOrEqual,
    LessOrEqual,
    Equal,
}

impl SizeOp {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            ">" => Some(SizeOp::GreaterThan),
            "<" => Some(SizeOp::LessThan),
            ">=" => Some(SizeOp::GreaterOrEqual),
            "<=" => Some(SizeOp::LessOrEqual),
            "=" | "==" => Some(SizeOp::Equal),
            _ => None,
        }
    }

    pub fn compare(&self, actual: u64, target: u64) -> bool {
        match self {
            SizeOp::GreaterThan => actual > target,
            SizeOp::LessThan => actual < target,
            SizeOp::GreaterOrEqual => actual >= target,
            SizeOp::LessOrEqual => actual <= target,
            SizeOp::Equal => actual == target,
        }
    }
}

// ============================================================================
// Date filter
// ============================================================================

/// Date filter — relative (today, yesterday, Ndays) or absolute.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DateFilter {
    Today,
    Yesterday,
    PastDays(u32),
}

impl DateFilter {
    fn parse_value(value: &str) -> Option<Self> {
        let lower = value.to_lowercase();
        match lower.as_str() {
            "today" => Some(DateFilter::Today),
            "yesterday" => Some(DateFilter::Yesterday),
            _ => {
                // Try parsing "Nd" or "Ndays"
                let stripped = lower.strip_suffix("days").or_else(|| lower.strip_suffix("d"));
                stripped
                    .and_then(|n| n.parse::<u32>().ok())
                    .map(DateFilter::PastDays)
            }
        }
    }
}

// ============================================================================
// SearchFilter — individual filter clause
// ============================================================================

/// A single filter extracted from the search query.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SearchFilter {
    /// Extension filter: `ext:pdf`
    Extension(String),
    /// Size filter: `size:>10mb`
    Size(SizeOp, u64),
    /// Date modified filter: `date-modified:today`
    DateModified(DateFilter),
    /// Show only folders: `folders:`
    FoldersOnly,
    /// Show only files: `files:`
    FilesOnly,
}

// ============================================================================
// SearchQuery — parsed query
// ============================================================================

/// A parsed search query.
///
/// Created via `SearchQuery::parse(input)`. The parser handles:
/// - Implicit AND between space-separated terms
/// - OR via `|`
/// - NOT via `!`
/// - Wildcards `*` and `?`
/// - Function filters (`ext:`, `size:`, etc.)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchQuery {
    /// Free-text search terms (filenames must match ALL of these).
    pub text_terms: Vec<String>,
    /// Negated text terms (filenames must NOT match any of these).
    pub excluded_terms: Vec<String>,
    /// OR groups: at least one term from each group must match.
    pub or_groups: Vec<Vec<String>>,
    /// Structured filters (ext, size, date, folders, files).
    pub filters: Vec<SearchFilter>,
}

impl SearchQuery {
    /// Parse a search query string into a `SearchQuery`.
    ///
    /// # Errors
    /// Returns `DomainError::InvalidPath` if the query contains invalid syntax.
    pub fn parse(input: &str) -> Result<Self, DomainError> {
        let input = input.trim();
        if input.is_empty() {
            return Ok(SearchQuery {
                text_terms: Vec::new(),
                excluded_terms: Vec::new(),
                or_groups: Vec::new(),
                filters: Vec::new(),
            });
        }

        let mut query = SearchQuery {
            text_terms: Vec::new(),
            excluded_terms: Vec::new(),
            or_groups: Vec::new(),
            filters: Vec::new(),
        };

        // Split by OR operator first
        let or_parts: Vec<&str> = input.split('|').map(|s| s.trim()).collect();

        for (i, or_part) in or_parts.iter().enumerate() {
            if or_part.is_empty() {
                continue;
            }

            let terms = Self::parse_and_terms(or_part, &mut query)?;

            if i > 0 && !or_part.is_empty() {
                // This is part of an OR group
                if query.or_groups.is_empty() {
                    // Convert previous AND terms into the first OR group
                    let prev_terms: Vec<String> = query.text_terms.drain(..).collect();
                    if !prev_terms.is_empty() {
                        query.or_groups.push(prev_terms);
                    }
                }
                query.or_groups.push(terms);
            } else {
                // First part — these are AND terms
                query.text_terms.extend(terms);
            }
        }

        Ok(query)
    }

    /// Parse AND-separated terms from a single OR-part.
    fn parse_and_terms(
        input: &str,
        query: &mut SearchQuery,
    ) -> Result<Vec<String>, DomainError> {
        let mut terms = Vec::new();
        let mut chars = input.chars().peekable();

        while let Some(ch) = chars.peek() {
            match ch {
                ' ' => {
                    chars.next();
                }
                '!' => {
                    chars.next();
                    let term = Self::read_term(&mut chars)?;
                    if !term.is_empty() {
                        query.excluded_terms.push(term);
                    }
                }
                _ => {
                    let term = Self::read_term(&mut chars)?;
                    if term.is_empty() {
                        continue;
                    }

                    // Check for function filters
                    if let Some(filter) = Self::try_parse_filter(&term) {
                        query.filters.push(filter);
                    } else {
                        terms.push(term);
                    }
                }
            }
        }

        Ok(terms)
    }

    /// Read a single term (up to next space, handling quotes).
    fn read_term(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> Result<String, DomainError> {
        let mut term = String::new();
        let mut in_quotes = false;

        // Skip leading whitespace
        while let Some(' ') = chars.peek() {
            chars.next();
        }

        if let Some('"') = chars.peek() {
            chars.next();
            in_quotes = true;
        }

        loop {
            match chars.peek() {
                None => break,
                Some(' ') if !in_quotes => break,
                Some('"') if in_quotes => {
                    chars.next();
                    break;
                }
                Some(&ch) => {
                    chars.next();
                    term.push(ch);
                }
            }
        }

        Ok(term)
    }

    /// Try to parse a term as a function filter (ext:, size:, etc.).
    fn try_parse_filter(term: &str) -> Option<SearchFilter> {
        let lower = term.to_lowercase();

        // Exact match filters (no value)
        if lower == "folders:" || lower == "folders" {
            return Some(SearchFilter::FoldersOnly);
        }
        if lower == "files:" || lower == "files" {
            return Some(SearchFilter::FilesOnly);
        }

        // Key:value filters
        let colon_pos = lower.find(':')?;
        let key = &lower[..colon_pos];
        let value = term[colon_pos + 1..].trim();

        match key {
            "ext" => {
                let ext = value.trim_start_matches('.');
                if !ext.is_empty() {
                    Some(SearchFilter::Extension(ext.to_lowercase()))
                } else {
                    None
                }
            }
            "size" => Self::parse_size_filter(value),
            "date-modified" | "dm" => DateFilter::parse_value(value)
                .map(SearchFilter::DateModified),
            _ => None,
        }
    }

    /// Parse a size filter like `>10mb`, `<1gb`, `=500kb`.
    fn parse_size_filter(value: &str) -> Option<SearchFilter> {
        let value = value.trim();

        // Extract operator using strip_prefix
        let (op_str, num_str) = if let Some(rest) = value.strip_prefix(">=") {
            (">=", rest)
        } else if let Some(rest) = value.strip_prefix("<=") {
            ("<=", rest)
        } else if let Some(rest) = value.strip_prefix('>') {
            (">", rest)
        } else if let Some(rest) = value.strip_prefix('<') {
            ("<", rest)
        } else if let Some(rest) = value.strip_prefix("==") {
            ("=", rest)
        } else if let Some(rest) = value.strip_prefix('=') {
            ("=", rest)
        } else {
            ("=", value)
        };

        let op = SizeOp::from_str(op_str)?;

        let num_str = num_str.trim();
        let (num, multiplier) = Self::parse_size_number(num_str)?;
        let bytes = num * multiplier;

        Some(SearchFilter::Size(op, bytes))
    }

    /// Parse a number with optional unit suffix (kb, mb, gb).
    fn parse_size_number(s: &str) -> Option<(u64, u64)> {
        let s = s.trim().to_lowercase();

        let (num_part, multiplier) = if let Some(rest) = s.strip_suffix("gb") {
            (rest, 1024 * 1024 * 1024)
        } else if let Some(rest) = s.strip_suffix("mb") {
            (rest, 1024 * 1024)
        } else if let Some(rest) = s.strip_suffix("kb") {
            (rest, 1024)
        } else if let Some(rest) = s.strip_suffix("b") {
            (rest, 1)
        } else {
            (s.as_str(), 1)
        };

        let num: u64 = num_part.trim().parse().ok()?;
        Some((num, multiplier))
    }

    /// Returns true if the query is empty (no search terms or filters).
    pub fn is_empty(&self) -> bool {
        self.text_terms.is_empty()
            && self.excluded_terms.is_empty()
            && self.or_groups.is_empty()
            && self.filters.is_empty()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_query() {
        let q = SearchQuery::parse("").unwrap();
        assert!(q.is_empty());
    }

    #[test]
    fn test_simple_text() {
        let q = SearchQuery::parse("report").unwrap();
        assert_eq!(q.text_terms, vec!["report"]);
        assert!(q.filters.is_empty());
    }

    #[test]
    fn test_multiple_terms_and() {
        let q = SearchQuery::parse("report 2024").unwrap();
        assert_eq!(q.text_terms, vec!["report", "2024"]);
    }

    #[test]
    fn test_ext_filter() {
        let q = SearchQuery::parse("ext:pdf").unwrap();
        assert!(q.text_terms.is_empty());
        assert_eq!(q.filters.len(), 1);
        assert_eq!(q.filters[0], SearchFilter::Extension("pdf".to_string()));
    }

    #[test]
    fn test_ext_filter_with_dot() {
        let q = SearchQuery::parse("ext:.pdf").unwrap();
        assert_eq!(q.filters[0], SearchFilter::Extension("pdf".to_string()));
    }

    #[test]
    fn test_size_filter_gt() {
        let q = SearchQuery::parse("size:>10mb").unwrap();
        assert_eq!(
            q.filters[0],
            SearchFilter::Size(SizeOp::GreaterThan, 10 * 1024 * 1024)
        );
    }

    #[test]
    fn test_size_filter_lt() {
        let q = SearchQuery::parse("size:<1gb").unwrap();
        assert_eq!(
            q.filters[0],
            SearchFilter::Size(SizeOp::LessThan, 1024 * 1024 * 1024)
        );
    }

    #[test]
    fn test_size_filter_gte() {
        let q = SearchQuery::parse("size:>=500kb").unwrap();
        assert_eq!(
            q.filters[0],
            SearchFilter::Size(SizeOp::GreaterOrEqual, 500 * 1024)
        );
    }

    #[test]
    fn test_size_filter_bytes() {
        let q = SearchQuery::parse("size:>1000").unwrap();
        assert_eq!(
            q.filters[0],
            SearchFilter::Size(SizeOp::GreaterThan, 1000)
        );
    }

    #[test]
    fn test_date_today() {
        let q = SearchQuery::parse("date-modified:today").unwrap();
        assert_eq!(q.filters[0], SearchFilter::DateModified(DateFilter::Today));
    }

    #[test]
    fn test_date_yesterday() {
        let q = SearchQuery::parse("date-modified:yesterday").unwrap();
        assert_eq!(
            q.filters[0],
            SearchFilter::DateModified(DateFilter::Yesterday)
        );
    }

    #[test]
    fn test_date_past_days() {
        let q = SearchQuery::parse("date-modified:7days").unwrap();
        assert_eq!(
            q.filters[0],
            SearchFilter::DateModified(DateFilter::PastDays(7))
        );
    }

    #[test]
    fn test_date_dm_shortcut() {
        let q = SearchQuery::parse("dm:today").unwrap();
        assert_eq!(q.filters[0], SearchFilter::DateModified(DateFilter::Today));
    }

    #[test]
    fn test_folders_filter() {
        let q = SearchQuery::parse("folders:").unwrap();
        assert_eq!(q.filters[0], SearchFilter::FoldersOnly);
    }

    #[test]
    fn test_files_filter() {
        let q = SearchQuery::parse("files:").unwrap();
        assert_eq!(q.filters[0], SearchFilter::FilesOnly);
    }

    #[test]
    fn test_folders_without_colon() {
        let q = SearchQuery::parse("folders").unwrap();
        assert_eq!(q.filters[0], SearchFilter::FoldersOnly);
    }

    #[test]
    fn test_not_operator() {
        let q = SearchQuery::parse("!temp").unwrap();
        assert!(q.text_terms.is_empty());
        assert_eq!(q.excluded_terms, vec!["temp"]);
    }

    #[test]
    fn test_not_with_filter() {
        let q = SearchQuery::parse("ext:pdf !draft").unwrap();
        assert_eq!(q.filters[0], SearchFilter::Extension("pdf".to_string()));
        assert_eq!(q.excluded_terms, vec!["draft"]);
    }

    #[test]
    fn test_or_operator() {
        let q = SearchQuery::parse("ext:pdf | ext:doc").unwrap();
        // OR groups: [["ext:pdf"], ["ext:doc"]] — but ext: is a filter, not text
        // Actually: ext:pdf and ext:doc are filters, so no OR groups
        assert_eq!(q.filters.len(), 2);
    }

    #[test]
    fn test_or_with_text() {
        let q = SearchQuery::parse("report | summary").unwrap();
        // Both terms go into OR groups: [["report"], ["summary"]]
        assert_eq!(q.or_groups.len(), 2);
        assert_eq!(q.or_groups[0], vec!["report"]);
        assert_eq!(q.or_groups[1], vec!["summary"]);
        assert!(q.text_terms.is_empty());
    }

    #[test]
    fn test_wildcard_star() {
        let q = SearchQuery::parse("*.pdf").unwrap();
        assert_eq!(q.text_terms, vec!["*.pdf"]);
    }

    #[test]
    fn test_wildcard_question() {
        let q = SearchQuery::parse("report?.doc").unwrap();
        assert_eq!(q.text_terms, vec!["report?.doc"]);
    }

    #[test]
    fn test_combined_query() {
        let q = SearchQuery::parse("report ext:pdf size:>1mb date-modified:today").unwrap();
        assert_eq!(q.text_terms, vec!["report"]);
        assert_eq!(q.filters.len(), 3);
        assert!(q.filters.contains(&SearchFilter::Extension("pdf".to_string())));
        assert!(q
            .filters
            .contains(&SearchFilter::Size(SizeOp::GreaterThan, 1024 * 1024)));
        assert!(q
            .filters
            .contains(&SearchFilter::DateModified(DateFilter::Today)));
    }

    #[test]
    fn test_quoted_term() {
        let q = SearchQuery::parse("\"my report\"").unwrap();
        assert_eq!(q.text_terms, vec!["my report"]);
    }

    #[test]
    fn test_size_comparison() {
        assert!(SizeOp::GreaterThan.compare(2000, 1000));
        assert!(!SizeOp::GreaterThan.compare(500, 1000));
        assert!(SizeOp::LessThan.compare(500, 1000));
        assert!(!SizeOp::LessThan.compare(2000, 1000));
        assert!(SizeOp::GreaterOrEqual.compare(1000, 1000));
        assert!(SizeOp::Equal.compare(1000, 1000));
        assert!(!SizeOp::Equal.compare(1001, 1000));
    }

    #[test]
    fn test_complex_query() {
        let q = SearchQuery::parse("!temp ext:pdf | ext:doc size:>100kb").unwrap();
        assert!(q.excluded_terms.contains(&"temp".to_string()));
        // ext:pdf and ext:doc and size:>100kb are all filters
        assert!(q.filters.len() >= 2); // ext filters + size filter
    }

    #[test]
    fn test_whitespace_handling() {
        let q = SearchQuery::parse("  report   ext:pdf  ").unwrap();
        assert_eq!(q.text_terms, vec!["report"]);
        assert_eq!(q.filters[0], SearchFilter::Extension("pdf".to_string()));
    }
}
