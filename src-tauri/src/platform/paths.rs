//! Cross-platform config directory resolution.
//!
//! Uses the `directories` crate to get platform-specific paths:
//! - Windows: `%APPDATA%/QuickSort/`
//! - Linux: `~/.config/QuickSort/`
//! - macOS: `~/Library/Application Support/QuickSort/`

use std::path::PathBuf;

/// Get the QuickSort config directory.
///
/// Returns the platform-specific configuration directory:
/// - Windows: `%APPDATA%/QuickSort/`
/// - Linux: `~/.config/QuickSort/`
/// - macOS: `~/Library/Application Support/QuickSort/`
///
/// Falls back to current directory if `directories` crate fails.
pub fn config_dir() -> PathBuf {
    directories::ProjectDirs::from("", "", "QuickSort")
        .map(|dirs| dirs.config_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from(".").join("QuickSort"))
}

/// Get the QuickSort data directory.
///
/// Returns the platform-specific data directory:
/// - Windows: `%LOCALAPPDATA%/QuickSort/`
/// - Linux: `~/.local/share/QuickSort/`
/// - macOS: `~/Library/Application Support/QuickSort/`
pub fn data_dir() -> PathBuf {
    directories::ProjectDirs::from("", "", "QuickSort")
        .map(|dirs| dirs.data_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from(".").join("QuickSort"))
}

/// Get the QuickSort cache directory.
///
/// Returns the platform-specific cache directory:
/// - Windows: `%LOCALAPPDATA%/QuickSort/cache/`
/// - Linux: `~/.cache/QuickSort/`
/// - macOS: `~/Library/Caches/QuickSort/`
#[allow(dead_code)]
pub fn cache_dir() -> PathBuf {
    directories::ProjectDirs::from("", "", "QuickSort")
        .map(|dirs| dirs.cache_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from(".").join("QuickSort").join("cache"))
}

/// Get the path to the folders.json config file.
pub fn folders_config_path() -> PathBuf {
    config_dir().join("folders.json")
}

/// Get the path to the settings.json config file.
pub fn settings_config_path() -> PathBuf {
    config_dir().join("settings.json")
}

/// Get the path to the operations.json file.
pub fn operations_path() -> PathBuf {
    data_dir().join("operations.json")
}

/// Get the path to the PID file (Windows only).
#[cfg(target_os = "windows")]
pub fn pid_file_path() -> PathBuf {
    config_dir().join("dll_owner.pid")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_dir_returns_valid_path() {
        let path = config_dir();
        assert!(path.to_string_lossy().contains("QuickSort"));
    }

    #[test]
    fn test_data_dir_returns_valid_path() {
        let path = data_dir();
        assert!(path.to_string_lossy().contains("QuickSort"));
    }

    #[test]
    fn test_cache_dir_returns_valid_path() {
        let path = cache_dir();
        assert!(path.to_string_lossy().contains("QuickSort"));
    }

    #[test]
    fn test_folders_config_path() {
        let path = folders_config_path();
        assert!(path.ends_with("folders.json"));
    }

    #[test]
    fn test_settings_config_path() {
        let path = settings_config_path();
        assert!(path.ends_with("settings.json"));
    }

    #[test]
    fn test_operations_path() {
        let path = operations_path();
        assert!(path.ends_with("operations.json"));
    }
}
