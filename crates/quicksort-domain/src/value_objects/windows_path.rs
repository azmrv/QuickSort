//! Absolute path value object – a validated, absolute filesystem path.
//!
//! `AbsolutePath` guarantees that the contained path is a valid, absolute
//! path on the current platform (e.g., `C:\folder\file.txt` or
//! `\\server\share\...` on Windows, `/home/user/file.txt` on Unix).
//! It is the only type allowed to cross domain boundaries as a file location.
//!
//! # Invariants
//! - The path is never empty.
//! - The path is absolute (starts with a drive letter / UNC prefix on Windows,
//!   or a root separator on Unix).
//! - Path traversal (`..`) is rejected.
//!
//! # Usage
//! Construction is fallible – use `AbsolutePath::new()` which validates the
//! input and returns a `DomainError` for invalid paths.  Once constructed,
//! the value can be used safely everywhere in the domain.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::{Path, PathBuf};

use crate::errors::DomainError;

// ---------------------------------------------------------------------------
// AbsolutePath
// ---------------------------------------------------------------------------

/// A validated, absolute filesystem path.
///
/// Works cross-platform: accepts Windows drive-letter paths (`C:\...`),
/// UNC paths (`\\server\share\...`), and Unix root-relative paths
/// (`/home/user/...`).
///
/// # Examples
/// ```rust
/// use quicksort_domain::value_objects::AbsolutePath;
/// let path = AbsolutePath::new("C:\\Users\\Me\\Documents").unwrap();
/// assert!(path.is_absolute());
/// assert_eq!(path.file_name(), Some("Documents"));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AbsolutePath(PathBuf);

// An empty path violates the type's invariant.  A reasonable default is
// the root of the current filesystem, which is always valid.
impl Default for AbsolutePath {
    fn default() -> Self {
        let current_dir = std::env::current_dir().unwrap_or_else(|_| {
            #[cfg(target_os = "windows")]
            {
                PathBuf::from("C:\\")
            }
            #[cfg(not(target_os = "windows"))]
            {
                PathBuf::from("/")
            }
        });
        let root = current_dir
            .ancestors()
            .last()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| {
                #[cfg(target_os = "windows")]
                {
                    PathBuf::from("C:\\")
                }
                #[cfg(not(target_os = "windows"))]
                {
                    PathBuf::from("/")
                }
            });
        // Ensure the root ends with a separator so AbsolutePath::new accepts it.
        let mut root_str = root.to_string_lossy().to_string();
        let separator = std::path::MAIN_SEPARATOR;
        if !root_str.ends_with(separator) && !root_str.ends_with('/') {
            root_str.push(separator);
        }
        AbsolutePath::new(&root_str).expect("Default AbsolutePath must be valid")
    }
}

impl AbsolutePath {
    /// Creates a new `AbsolutePath` from a string, validating the format.
    ///
    /// # Validation rules
    /// - The path must not be empty.
    /// - The path must not contain `..` components (path traversal).
    /// - On Windows: forward slashes are normalised to backslashes; the path
    ///   must start with a drive letter (`C:\`), UNC prefix (`\\`), or root.
    /// - On Unix: backslashes are normalised to forward slashes; the path
    ///   must start with `/`.
    /// - Platform-agnostic: uses `Path::is_absolute()` as the primary check.
    ///
    /// # Errors
    /// Returns `DomainError::EmptyPath`, `DomainError::PathTraversalAttempt`,
    /// or `DomainError::InvalidPath` if the input does not meet the requirements.
    pub fn new(path: &str) -> Result<Self, DomainError> {
        // Normalise separators: forward slash → backslash only on Windows,
        // backslash → forward slash only on Unix.
        let s = if cfg!(target_os = "windows") {
            path.replace('/', "\\")
        } else {
            path.replace('\\', "/")
        };

        // Reject empty strings immediately.
        if s.is_empty() {
            return Err(DomainError::EmptyPath);
        }

        // Block path traversal attempts — reject ".." components.
        if s.contains("..") {
            return Err(DomainError::PathTraversalAttempt(s));
        }

        // Must be absolute — platform-agnostic check.
        let path_obj = Path::new(&s);
        if !path_obj.is_absolute() {
            return Err(DomainError::InvalidPath(
                "Path must be absolute".to_string(),
            ));
        }

        Ok(Self(PathBuf::from(s)))
    }

    // This method is kept for backward compatibility but marked deprecated.
    // New code should use `AbsolutePath::new()` instead.
    #[deprecated(
        since = "0.2.0",
        note = "Use AbsolutePath::new() for validated construction"
    )]
    pub fn try_from_str(path: &str) -> Result<Self, PathConversionError> {
        let inner = PathBuf::from(path);
        Ok(Self(inner))
    }

    /// Returns a clone of the inner `PathBuf`.
    pub fn to_path_buf(&self) -> PathBuf {
        self.0.clone()
    }

    /// Returns the path as a string slice, if it is valid UTF-8.
    pub fn as_str(&self) -> Option<&str> {
        self.0.as_os_str().to_str()
    }

    /// Returns the file name component (e.g., `file.txt` for `C:\dir\file.txt`
    /// or `/home/user/file.txt`).
    pub fn file_name(&self) -> Option<&str> {
        self.0.file_name().and_then(|s| s.to_str())
    }

    /// Returns the file extension, if any, without the leading dot.
    pub fn extension(&self) -> Option<&str> {
        self.0.extension().and_then(|e| e.to_str())
    }

    /// Returns the parent directory, if any.
    pub fn parent(&self) -> Option<AbsolutePath> {
        self.0.parent().map(|p| AbsolutePath(p.to_path_buf()))
    }

    /// Checks whether the path is absolute (always `true` for validated paths).
    pub fn is_absolute(&self) -> bool {
        self.0.is_absolute()
    }

    /// Joins a path component to this path.
    ///
    /// # Example
    /// ```rust
    /// use quicksort_domain::value_objects::AbsolutePath;
    /// let base = AbsolutePath::new("C:\\Users").unwrap();
    /// let full = base.join("Documents");
    /// assert_eq!(full.to_string(), "C:\\Users\\Documents");
    /// ```
    pub fn join(&self, component: impl AsRef<str>) -> AbsolutePath {
        let joined = self.0.join(component.as_ref());
        AbsolutePath(joined)
    }

    /// Returns the drive letter portion (e.g., `"C:"` for `C:\folder`).
    ///
    /// Only available on Windows. On other platforms, this method does
    /// not exist — use `root()` instead to get the root component.
    #[cfg(target_os = "windows")]
    pub fn drive(&self) -> Option<String> {
        self.as_str().map(|s| s.chars().take(2).collect())
    }

    /// Returns the root component of the path (e.g., `"C:\\"` for
    /// `C:\folder\file`, or `"/"` for `/home/user/file`).
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

    /// Checks whether the path refers to a directory (based on a trailing separator).
    pub fn is_directory(&self) -> bool {
        self.as_str()
            .map(|s| s.ends_with('\\') || s.ends_with('/'))
            .unwrap_or(false)
    }

    /// Checks whether the path is a root (e.g., `C:\` on Windows, `/` on Unix).
    /// A root path has no `Normal` components — only Prefix, RootDir, etc.
    pub fn is_root(&self) -> bool {
        use std::path::Component;
        !self
            .0
            .components()
            .any(|c| matches!(c, Component::Normal(_)))
    }

    /// Returns the string slice without checking (for internal use).
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

impl fmt::Display for AbsolutePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.display())
    }
}

impl From<PathBuf> for AbsolutePath {
    /// Converts a `PathBuf` into an `AbsolutePath` without validation.
    ///
    /// # Safety
    /// This bypasses all validation checks (empty, absolute, traversal).
    /// Only use with trusted data (e.g., internal domain operations).
    /// For external/untrusted input, always use `AbsolutePath::new()`.
    fn from(path: PathBuf) -> Self {
        Self(path)
    }
}

impl AsRef<Path> for AbsolutePath {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl AsRef<PathBuf> for AbsolutePath {
    fn as_ref(&self) -> &PathBuf {
        &self.0
    }
}

impl std::ops::Deref for AbsolutePath {
    type Target = PathBuf;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// ---------------------------------------------------------------------------
// Backward compatibility type alias
// ---------------------------------------------------------------------------

/// Type alias for backward compatibility.
///
/// New code should use `AbsolutePath` directly.
pub type WindowsPath = AbsolutePath;

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

    // -- Windows-style paths (still valid on Windows) --

    #[test]
    fn test_create_valid_drive_path() {
        let path = AbsolutePath::new("C:\\folder\\file.txt").unwrap();
        assert_eq!(path.to_string(), "C:\\folder\\file.txt");
    }

    #[test]
    fn test_create_unc_path() {
        let path = AbsolutePath::new("\\\\server\\share\\file.txt").unwrap();
        assert!(path.is_absolute());
    }

    #[test]
    fn test_reject_empty() {
        assert!(matches!(AbsolutePath::new(""), Err(DomainError::EmptyPath)));
    }

    #[test]
    fn test_reject_relative() {
        assert!(matches!(
            AbsolutePath::new("folder\\file.txt"),
            Err(DomainError::InvalidPath(_))
        ));
    }

    #[test]
    fn test_reject_path_traversal() {
        assert!(matches!(
            AbsolutePath::new("C:\\folder\\..\\..\\Windows"),
            Err(DomainError::PathTraversalAttempt(_))
        ));
    }

    #[test]
    fn test_reject_path_traversal_encoded() {
        // Even with forward slashes (normalised to backslashes on Windows)
        assert!(matches!(
            AbsolutePath::new("C:/folder/../Windows"),
            Err(DomainError::PathTraversalAttempt(_))
        ));
    }

    #[test]
    fn test_normalise_forward_slashes() {
        let path = AbsolutePath::new("C:/folder/file.txt").unwrap();
        assert_eq!(path.to_string(), "C:\\folder\\file.txt");
    }

    #[test]
    fn test_file_name() {
        let path = AbsolutePath::new("C:\\folder\\file.txt").unwrap();
        assert_eq!(path.file_name(), Some("file.txt"));
    }

    #[test]
    fn test_extension() {
        let path = AbsolutePath::new("C:\\folder\\file.txt").unwrap();
        assert_eq!(path.extension(), Some("txt"));
    }

    #[test]
    fn test_parent() {
        let path = AbsolutePath::new("C:\\folder\\subfolder\\file.txt").unwrap();
        let parent = path.parent().unwrap();
        assert_eq!(parent.to_string(), "C:\\folder\\subfolder");
    }

    #[test]
    fn test_join() {
        let path = AbsolutePath::new("C:\\folder").unwrap();
        let joined = path.join("subfolder");
        assert_eq!(joined.to_string(), "C:\\folder\\subfolder");
    }

    #[test]
    fn test_is_root() {
        let root = AbsolutePath::new("C:\\").unwrap();
        assert!(root.is_root());
        let not_root = AbsolutePath::new("C:\\folder").unwrap();
        assert!(!not_root.is_root());
    }

    #[test]
    fn test_drive() {
        let path = AbsolutePath::new("D:\\folder\\file.txt").unwrap();
        assert_eq!(path.drive(), Some("D:".to_string()));
    }

    #[test]
    fn test_default_is_valid() {
        let default_path = AbsolutePath::default();
        assert!(default_path.is_absolute());
        assert!(!default_path.to_string().is_empty());
    }

    // -- Unix-style paths (valid on all platforms via Path::is_absolute) --

    #[test]
    fn test_create_valid_unix_path() {
        let path = AbsolutePath::new("/home/user/documents").unwrap();
        assert!(path.is_absolute());
    }

    #[test]
    fn test_unix_root_is_root() {
        let path = AbsolutePath::new("/").unwrap();
        assert!(path.is_root());
    }

    #[test]
    fn test_unix_file_name() {
        let path = AbsolutePath::new("/home/user/file.txt").unwrap();
        assert_eq!(path.file_name(), Some("file.txt"));
    }

    #[test]
    fn test_unix_parent() {
        let path = AbsolutePath::new("/home/user/file.txt").unwrap();
        let parent = path.parent().unwrap();
        assert_eq!(parent.to_string(), "/home/user");
    }

    #[test]
    fn test_unix_join() {
        let path = AbsolutePath::new("/home/user").unwrap();
        let joined = path.join("documents");
        assert_eq!(joined.to_string(), "/home/user/documents");
    }

    #[test]
    fn test_unix_reject_relative() {
        assert!(matches!(
            AbsolutePath::new("home/user/file.txt"),
            Err(DomainError::InvalidPath(_))
        ));
    }

    #[test]
    fn test_unix_reject_path_traversal() {
        assert!(matches!(
            AbsolutePath::new("/home/user/../../etc/passwd"),
            Err(DomainError::PathTraversalAttempt(_))
        ));
    }

    // -- Backward compatibility --

    #[test]
    fn test_windows_path_alias() {
        let path = WindowsPath::new("C:\\folder\\file.txt").unwrap();
        assert!(path.is_absolute());
        assert_eq!(path.file_name(), Some("file.txt"));
    }
}
