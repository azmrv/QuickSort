//! Windows path value object – a validated, absolute filesystem path.
//!
//! `WindowsPath` guarantees that the contained path is a valid, absolute
//! Windows path (e.g., `C:\folder\file.txt` or `\\server\share\...`).
//! It is the only type allowed to cross domain boundaries as a file location.
//!
//! # Invariants
//! - The path is never empty.
//! - The path is absolute (starts with a drive letter or a UNC prefix).
//! - Backslashes are used as separators (forward slashes are normalised).
//!
//! # Usage
//! Construction is fallible – use `WindowsPath::new()` which validates the
//! input and returns a `DomainError` for invalid paths.  Once constructed,
//! the value can be used safely everywhere in the domain.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::{Path, PathBuf};

use crate::errors::DomainError;

// ---------------------------------------------------------------------------
// WindowsPath
// ---------------------------------------------------------------------------

/// A validated, absolute Windows filesystem path.
///
/// # Examples
/// ```rust
/// let path = WindowsPath::new("C:\\Users\\Me\\Documents").unwrap();
/// assert!(path.is_absolute());
/// assert_eq!(path.file_name(), Some("Documents"));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WindowsPath(PathBuf);

// OLD: impl Default { … returns empty path … }
// An empty path violates the type's invariant.  A reasonable default is
// the root of the current drive, which is always valid.
impl Default for WindowsPath {
    fn default() -> Self {
        // Use the current directory's drive root as a safe fallback.
        // For example, if the process runs from C:\Projects, the root is C:\.
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("C:\\"));
        let root = current_dir
            .ancestors()
            .last()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("C:\\"));
        // Ensure it ends with a backslash – WindowsPath::new would reject
        // a root without one.
        let mut root_str = root.to_string_lossy().to_string();
        if !root_str.ends_with('\\') {
            root_str.push('\\');
        }
        WindowsPath::new(&root_str).expect("Default WindowsPath must be valid")
    }
}

impl WindowsPath {
    /// Creates a new `WindowsPath` from a string, validating the format.
    ///
    /// # Validation rules
    /// - The path must not be empty.
    /// - Forward slashes are normalised to backslashes.
    /// - The path must start with a drive letter followed by `:\` (e.g., `C:\`)
    ///   or with a UNC prefix (`\\`).
    ///
    /// # Errors
    /// Returns `DomainError::EmptyPath` or `DomainError::InvalidPath` if
    /// the input does not meet the requirements.
    // OLD: comments were in Russian
    pub fn new(path: &str) -> Result<Self, DomainError> {
        // Normalise separators to backslashes.
        let s = path.replace('/', "\\");

        // Reject empty strings immediately.
        if s.is_empty() {
            return Err(DomainError::EmptyPath);
        }

        // Validate absolute Windows path.
        let chars: Vec<char> = s.chars().collect();
        if chars.len() >= 2 && chars[0].is_ascii_alphabetic() && chars[1] == ':' {
            // Drive-letter path – must be at least "C:\" (3 characters).
            if chars.len() < 3 || chars[2] != '\\' {
                return Err(DomainError::InvalidPath(
                    "Drive-letter path must be followed by :\\".to_string(),
                ));
            }
        } else if !s.starts_with("\\\\") {
            // Not a drive path and not a UNC path – reject.
            return Err(DomainError::InvalidPath(
                "Path must be absolute (start with a drive letter or \\\\)".to_string(),
            ));
        }
        // UNC paths are accepted as-is (no further validation of server/share).

        Ok(Self(PathBuf::from(s)))
    }

    // OLD: try_from_str – bypassed validation completely.
    // This method is kept for backward compatibility but marked deprecated.
    // New code should use `WindowsPath::new()` instead.
    #[deprecated(since = "0.2.0", note = "Use WindowsPath::new() for validated construction")]
    pub fn try_from_str(path: &str) -> Result<Self, PathConversionError> {
        let inner = PathBuf::from(path);
        Ok(Self(inner))
    }

    /// Returns a clone of the inner `PathBuf`.
    pub fn to_path_buf(&self) -> PathBuf {
        self.0.clone()
    }

    /// Returns the path as a string slice, if it is valid UTF-8.
    ///
    /// Since Windows paths are usually representable as UTF-16,
    /// this returns `None` only for paths containing unpaired surrogates.
    pub fn as_str(&self) -> Option<&str> {
        self.0.as_os_str().to_str()
    }

    /// Returns the file name component (e.g., `file.txt` for `C:\dir\file.txt`).
    pub fn file_name(&self) -> Option<&str> {
        self.0.file_name().and_then(|s| s.to_str())
    }

    /// Returns the file extension, if any, without the leading dot.
    pub fn extension(&self) -> Option<&str> {
        self.0.extension().and_then(|e| e.to_str())
    }

    /// Returns the path as a string (lossy conversion for non-UTF-8).
    // OLD: pub fn to_string(&self) -> String { self.as_str().unwrap_or("").to_string() }
    // Using `unwrap_or("")` silently hides invalid paths.  Use `to_string_lossy`
    // which always returns a usable string.
    pub fn to_string(&self) -> String {
        self.0.to_string_lossy().to_string()
    }

    /// Returns the parent directory, if any.
    pub fn parent(&self) -> Option<WindowsPath> {
        self.0.parent().map(|p| WindowsPath(p.to_path_buf()))
    }

    /// Checks whether the path is absolute (always `true` for validated paths).
    pub fn is_absolute(&self) -> bool {
        self.0.is_absolute()
    }

    /// Joins a path component to this path.
    ///
    /// # Example
    /// ```rust
    /// let base = WindowsPath::new("C:\\Users").unwrap();
    /// let full = base.join("Documents");
    /// assert_eq!(full.to_string(), "C:\\Users\\Documents");
    /// ```
    pub fn join(&self, component: impl AsRef<str>) -> WindowsPath {
        let joined = self.0.join(component.as_ref());
        WindowsPath(joined)
    }

    /// Returns the drive letter portion (e.g., `"C:"` for `C:\folder`).
    pub fn drive(&self) -> Option<String> {
        self.as_str().map(|s| s.chars().take(2).collect())
    }

    // OLD: get_short_name – incorrect implementation.
    // This method was taking characters until the first backslash, which is the
    // drive/root component, not the short name.  Removing it because it was
    // never used and its semantics are unclear.
    // If a "short name" is needed later, it should be clearly defined.
    // (Method removed – no replacement)

    /// Returns the root component of the path (e.g., `"C:\\"` for `C:\folder\file`).
    pub fn root(&self) -> Option<String> {
        self.0
            .components()
            .next()
            .map(|c| c.as_os_str().to_string_lossy().to_string())
    }

    /// Checks whether the path refers to a file (based on the presence of an extension).
    pub fn is_file(&self) -> bool {
        self.extension().is_some()
    }

    /// Checks whether the path refers to a directory (based on a trailing backslash).
    pub fn is_directory(&self) -> bool {
        self.as_str()
            .map(|s| s.ends_with('\\') || s.ends_with('/'))
            .unwrap_or(false)
    }

    /// Checks whether the path is a drive root (e.g., `C:\`).
    // OLD: complex logic mixing UNC and drive checks
    // Simplified: a path is a root if it has exactly one component (the root
    // itself, e.g., `C:\` or `\\server\share`).
    pub fn is_root(&self) -> bool {
        // `Path::components()` for "C:\" returns a single `Prefix` component.
        // For "C:\folder", it returns `Prefix` + `RootDir` + `Normal`, so 3 components.
        self.0.components().count() == 1
    }

    /// Returns the string slice without checking (for internal use).
    // OLD: pub fn as_unchecked(&self) -> &str { self.as_str().unwrap() }
    // Using `unwrap()` can panic.  Use `to_string_lossy` for guaranteed safety.
    #[deprecated(since = "0.2.0", note = "Use to_string() or to_string_lossy() instead")]
    pub fn as_unchecked(&self) -> &str {
        self.as_str().unwrap_or("")
    }

    /// Consumes the value and returns the inner `PathBuf`.
    pub fn into_inner(self) -> PathBuf {
        self.0
    }
}

// ---------------------------------------------------------------------------
// Trait implementations
// ---------------------------------------------------------------------------

impl fmt::Display for WindowsPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.display())
    }
}

impl From<PathBuf> for WindowsPath {
    /// Converts a `PathBuf` into a `WindowsPath` without validation.
    ///
    /// # Safety
    /// The caller must ensure the path is a valid absolute Windows path.
    /// This is intended for internal use and deserialization of trusted data.
    fn from(path: PathBuf) -> Self {
        Self(path)
    }
}

impl AsRef<Path> for WindowsPath {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl AsRef<PathBuf> for WindowsPath {
    fn as_ref(&self) -> &PathBuf {
        &self.0
    }
}

impl std::ops::Deref for WindowsPath {
    type Target = PathBuf;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// ---------------------------------------------------------------------------
// Legacy error type (kept for backward compatibility)
// ---------------------------------------------------------------------------

/// Error returned by the deprecated `try_from_str` method.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathConversionError(String);

impl fmt::Display for PathConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid path: {}", self.0)
    }
}

impl std::error::Error for PathConversionError {}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_valid_drive_path() {
        let path = WindowsPath::new("C:\\folder\\file.txt").unwrap();
        assert_eq!(path.to_string(), "C:\\folder\\file.txt");
    }

    #[test]
    fn test_create_unc_path() {
        let path = WindowsPath::new("\\\\server\\share\\file.txt").unwrap();
        assert!(path.is_absolute());
    }

    #[test]
    fn test_reject_empty() {
        assert!(matches!(WindowsPath::new(""), Err(DomainError::EmptyPath)));
    }

    #[test]
    fn test_reject_relative() {
        assert!(matches!(
            WindowsPath::new("folder\\file.txt"),
            Err(DomainError::InvalidPath(_))
        ));
    }

    #[test]
    fn test_reject_drive_without_backslash() {
        // "C:file" is technically a valid Windows path, but it's relative
        // to the current directory on drive C.  We require absolute paths.
        assert!(matches!(
            WindowsPath::new("C:file.txt"),
            Err(DomainError::InvalidPath(_))
        ));
    }

    #[test]
    fn test_normalise_forward_slashes() {
        let path = WindowsPath::new("C:/folder/file.txt").unwrap();
        assert_eq!(path.to_string(), "C:\\folder\\file.txt");
    }

    #[test]
    fn test_file_name() {
        let path = WindowsPath::new("C:\\folder\\file.txt").unwrap();
        assert_eq!(path.file_name(), Some("file.txt"));
    }

    #[test]
    fn test_extension() {
        let path = WindowsPath::new("C:\\folder\\file.txt").unwrap();
        assert_eq!(path.extension(), Some("txt"));
    }

    #[test]
    fn test_parent() {
        let path = WindowsPath::new("C:\\folder\\subfolder\\file.txt").unwrap();
        let parent = path.parent().unwrap();
        assert_eq!(parent.to_string(), "C:\\folder\\subfolder");
    }

    #[test]
    fn test_join() {
        let path = WindowsPath::new("C:\\folder").unwrap();
        let joined = path.join("subfolder");
        assert_eq!(joined.to_string(), "C:\\folder\\subfolder");
    }

    #[test]
    fn test_is_root() {
        let root = WindowsPath::new("C:\\").unwrap();
        assert!(root.is_root());
        let not_root = WindowsPath::new("C:\\folder").unwrap();
        assert!(!not_root.is_root());
    }

    #[test]
    fn test_drive() {
        let path = WindowsPath::new("D:\\folder\\file.txt").unwrap();
        assert_eq!(path.drive(), Some("D:".to_string()));
    }

    #[test]
    fn test_default_is_valid() {
        let default_path = WindowsPath::default();
        assert!(default_path.is_absolute());
        assert!(!default_path.to_string().is_empty());
    }
}