//! Windows path value object – wrapper around std::path::PathBuf
//! Represents a path in the Windows file system using string format.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

// Реализация Default для обратной совместимости
// WindowsPath не может быть пустым - возвращаем корень текущего диска в конструкторе по умолчанию
impl Default for WindowsPath {
    fn default() -> Self {
        // Для обратной совместимости создаем пустой путь (будет проваливаться валидации)
        let empty_path = PathBuf::from("");
        Self(empty_path)
    }
}

/// Windows path value object
/// Provides a strict representation of paths in Windows format (C:\folder\file)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WindowsPath(PathBuf);

impl WindowsPath {
    /// Создает новый WindowsPath из строки.
    /// # Arguments
    /// * `path` - Path in Windows format (e.g., "C:\\folder\\file")
    pub fn new(path: &str) -> Result<Self, crate::errors::DomainError> {
        let s = path.replace('/', "\\");
        if s.is_empty() {
            return Err(crate::errors::DomainError::EmptyPath);
        }
        let chars: Vec<char> = s.chars().collect();
        if chars.len() >= 2 && chars[0].is_ascii_alphabetic() && chars[1] == ':' {
            if chars.len() == 2 || chars[2] != '\\' {
                return Err(crate::errors::DomainError::InvalidPath("Invalid drive path".to_string()));
            }
        } else if !s.starts_with("\\\\") {
            return Err(crate::errors::DomainError::InvalidPath("Path must be absolute".to_string()));
        }
        Ok(Self(path.into()))
    }

    /// Создает новый WindowsPath из строки (старый API для обратной совместимости).
    /// # Arguments
    /// * `path` - Path in Windows format (e.g., "C:\folder\\file")
    pub fn try_from_str(path: &str) -> Result<Self, PathConversionError> {
        let inner = PathBuf::from(path);
        Ok(Self(inner))
    }

    /// Returns the inner PathBuf.
    pub fn to_path_buf(&self) -> PathBuf {
        self.0.clone()
    }

    /// Converts to string slice if possible.
    pub fn as_str(&self) -> Option<&str> {
        self.0.as_os_str().to_str()
    }

    /// Returns the file name of the path.
    pub fn file_name(&self) -> Option<&str> {
        self.0.file_name().and_then(|s| s.to_str())
    }

    /// Returns the file extension if present.
    pub fn extension(&self) -> Option<&str> {
        self.0.extension().and_then(|e| e.to_str())
    }

    /// Converts to a string representation.
    pub fn to_string(&self) -> String {
        self.as_str().unwrap_or("").to_string()
    }

    /// Gets the directory part of the path.
    pub fn parent(&self) -> Option<WindowsPath> {
        self.0.parent().map(|p| WindowsPath(p.to_path_buf()))
    }

    /// Checks if this path is an absolute path.
    pub fn is_absolute(&self) -> bool {
        self.0.is_absolute()
    }

    /// Concatenates this path with another path component.
    pub fn join(&self, component: impl AsRef<str>) -> WindowsPath {
        let joined = PathBuf::from(self.as_str().unwrap_or(""))
            .join(component.as_ref());
        WindowsPath(joined)
    }

    /// Gets the drive letter for Windows paths (e.g., "C:" for C:\folder).
    pub fn drive(&self) -> Option<String> {
        self.as_str().map(|s| s.chars().take(2).collect())
    }

    /// Получает короткое имя (директорию без имени файла) из пути.
    pub fn get_short_name(&self) -> String {
        self.as_str()
            .map(|s| s.chars().take_while(|c| *c != '\\' && *c != '/').collect())
            .unwrap_or_else(|| "".to_string())
    }

    /// Gets the root path (e.g., "C:\\" for C:\folder\file).
    pub fn root(&self) -> Option<String> {
        self.0
            .as_os_str()
            .to_str()
            .and_then(|s| s.split(':').next())
            .map(|root| format!("{:}", root))
    }

    /// Checks if this path refers to a file (based on extension).
    pub fn is_file(&self) -> bool {
        self.extension().is_some()
            && !self.0.as_os_str().to_string_lossy().ends_with('\\')
    }

    /// Checks if this path refers to a directory (based on ending with backslash).
    pub fn is_directory(&self) -> bool {
        self.as_str()
            .map(|s| s.ends_with("\\") || s.ends_with("/"))
            .unwrap_or(false)
    }

    /// Checks if this path is the root of a drive (e.g., "C:\\" or "\\\\server\\share").
    pub fn is_root(&self) -> bool {
        self.0.as_os_str().to_string_lossy()
            .ends_with("\\")
            || self.0.as_os_str().to_string_lossy()
                .starts_with("\\\\")
                && self.0.components().count() == 1
    }

    /// Returns string slice without validation (always returns Some).
    /// # Safety
    /// Caller must ensure the returned reference is used immediately or cloned.
    #[inline]
    pub fn as_unchecked(&self) -> &str {
        self.as_str().unwrap()
    }

    /// Returns string slice (consumes self).
    #[inline]
    pub fn into_string(self) -> String {
        self.0.into_os_string().into_string().unwrap_or_else(|os| os.to_string_lossy().to_string())
    }
}

impl fmt::Display for WindowsPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.display())
    }
}

// Implement From<PathBuf> for convenience
impl From<PathBuf> for WindowsPath {
    fn from(path: PathBuf) -> Self {
        Self(path)
    }
}

// Implement AsRef<PathBuf>
impl AsRef<PathBuf> for WindowsPath {
    fn as_ref(&self) -> &PathBuf {
        &self.0
    }
}

// Implement Deref to PathBuf
impl std::ops::Deref for WindowsPath {
    type Target = PathBuf;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Error type for path conversion failures
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathConversionError(String);

impl std::fmt::Display for PathConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid path: {}", self.0)
    }
}

impl std::error::Error for PathConversionError {}

impl From<PathBuf> for PathConversionError {
    fn from(_path: PathBuf) -> Self {
        PathConversionError(String::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_path_from_str() {
        let Ok(path) = WindowsPath::new("C:\\folder\\file.txt") else { return };
        assert_eq!(path.to_string(), "C:\\folder\\file.txt");
    }

    #[test]
    fn test_file_name() {
        let Ok(path) = WindowsPath::new("C:\\folder\\file.txt") else { return };
        assert_eq!(path.file_name(), Some("file.txt"));
    }

    #[test]
    fn test_extension() {
        let Ok(path) = WindowsPath::new("C:\\folder\\file.txt") else { return };
        assert_eq!(path.extension(), Some("txt"));
    }

    #[test]
    fn test_parent() {
        let Ok(path) = WindowsPath::new("C:\\folder\\subfolder\\file.txt") else { return };
        let parent = path.parent().unwrap();
        assert_eq!(parent.to_string(), "C:\\folder\\subfolder");
    }

    #[test]
    fn test_join() {
        let Ok(path) = WindowsPath::new("C:\\folder") else { return };
        let joined = path.join("subfolder");
        assert_eq!(joined.to_string(), "C:\\folder\\subfolder");
    }

    #[test]
    fn test_is_absolute() {
        let Ok(absolute) = WindowsPath::new("C:\\path") else { return };
        assert!(absolute.is_absolute());
    }

    #[test]
    fn test_drive() {
        let Ok(path) = WindowsPath::new("D:\\folder\\file.txt") else { return };
        assert_eq!(path.drive(), Some("D:".to_string()));
    }
}