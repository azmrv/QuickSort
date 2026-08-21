//! Plugin system traits and types.
//!
//! # Design Decisions
//! - Follows ADR-015: Plugin System Architecture
//! - Supports WCX (archive), WDX (content), WFX (filesystem), WLX (lister) plugins
//! - Plugins are loaded dynamically via DLL
//! - Domain traits define abstract interfaces; Infrastructure provides implementations

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Plugin type enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginType {
    /// Packer/Unpacker (archives) - WCX format
    Archive,
    /// Content metadata columns - WDX format
    Content,
    /// Virtual file systems - WFX format
    FileSystem,
    /// File viewers/listers - WLX format
    Lister,
    /// QuickSort-native plugin
    Native,
}

impl std::fmt::Display for PluginType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginType::Archive => write!(f, "archive"),
            PluginType::Content => write!(f, "content"),
            PluginType::FileSystem => write!(f, "filesystem"),
            PluginType::Lister => write!(f, "lister"),
            PluginType::Native => write!(f, "native"),
        }
    }
}

/// Plugin capabilities (derived from WCX GetPackerCaps).
#[derive(Debug, Clone, Default)]
pub struct PluginCapabilities {
    pub can_create: bool,
    pub can_modify: bool,
    pub supports_multiple: bool,
    pub can_delete: bool,
    pub has_options: bool,
    pub supports_mempack: bool,
    pub detect_by_content: bool,
    pub supports_search: bool,
    pub supports_encrypt: bool,
}

/// Plugin information for registry and UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub plugin_type: PluginType,
    pub source: String,
    pub path: String,
    pub enabled: bool,
    pub extensions: Vec<String>,
    pub settings: serde_json::Value,
}

/// Plugin configuration from manifest or registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub id: String,
    pub enabled: bool,
    pub settings: serde_json::Value,
}

/// Error type for plugin operations.
#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("Plugin not found: {0}")]
    NotFound(String),

    #[error("Plugin load failed: {0}")]
    LoadFailed(String),

    #[error("Plugin initialization failed: {0}")]
    InitFailed(String),

    #[error("Plugin operation failed: {0}")]
    OperationFailed(String),

    #[error("Plugin incompatible: {0}")]
    Incompatible(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Archive entry within an archive file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveEntry {
    pub path: String,
    pub size: u64,
    pub compressed_size: Option<u64>,
    pub is_directory: bool,
    pub modified_at: Option<String>,
}

/// Base trait for all QuickSort plugins.
pub trait Plugin: Send + Sync {
    /// Unique identifier (e.g., "com.quicksort.archive.7z").
    fn id(&self) -> &str;

    /// Human-readable name.
    fn name(&self) -> &str;

    /// Plugin version.
    fn version(&self) -> &str;

    /// Plugin type.
    fn plugin_type(&self) -> PluginType;

    /// Get plugin capabilities.
    fn capabilities(&self) -> PluginCapabilities {
        PluginCapabilities::default()
    }

    /// Get supported file extensions.
    fn supported_extensions(&self) -> Vec<String>;

    /// Initialize the plugin.
    fn initialize(&mut self, config: &PluginConfig) -> Result<(), PluginError>;

    /// Shutdown the plugin.
    fn shutdown(&mut self) -> Result<(), PluginError>;
}

/// Archive handling plugin trait.
pub trait ArchivePlugin: Plugin {
    /// Check if this plugin can handle the given file extension.
    fn can_handle(&self, extension: &str) -> bool;

    /// List contents of an archive.
    fn list_contents(&self, archive_path: &Path) -> Result<Vec<ArchiveEntry>, PluginError>;

    /// Extract a file from an archive.
    fn extract_file(
        &self,
        archive_path: &Path,
        entry_path: &str,
        output_path: &Path,
    ) -> Result<(), PluginError>;

    /// Add a file to an archive.
    fn add_file(
        &self,
        archive_path: &Path,
        file_path: &Path,
        entry_name: &str,
    ) -> Result<(), PluginError>;

    /// Create a new archive.
    fn create_archive(
        &self,
        archive_path: &Path,
        files: &[PathBuf],
    ) -> Result<(), PluginError>;
}

/// Content metadata plugin trait.
pub trait ContentPlugin: Plugin {
    /// Get supported metadata field names.
    fn supported_fields(&self) -> Vec<String>;

    /// Extract metadata from a file.
    fn extract_metadata(
        &self,
        file_path: &Path,
        field: &str,
    ) -> Result<Option<String>, PluginError>;
}

/// File system plugin trait.
pub trait FileSystemPlugin: Plugin {
    /// List files in a virtual directory.
    fn list_files(&self, path: &str) -> Result<Vec<ArchiveEntry>, PluginError>;

    /// Download a file from the virtual filesystem.
    fn get_file(
        &self,
        remote_path: &str,
        local_path: &Path,
    ) -> Result<(), PluginError>;

    /// Upload a file to the virtual filesystem.
    fn put_file(
        &self,
        local_path: &Path,
        remote_path: &str,
    ) -> Result<(), PluginError>;
}

/// Lister plugin trait.
pub trait ListerPlugin: Plugin {
    /// Check if this plugin can handle the given file type.
    fn can_handle(&self, file_path: &Path) -> bool;

    /// Get the plugin's window handle for preview.
    fn load_preview(
        &self,
        parent_hwnd: isize,
        file_path: &Path,
    ) -> Result<isize, PluginError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_type_display() {
        assert_eq!(PluginType::Archive.to_string(), "archive");
        assert_eq!(PluginType::Content.to_string(), "content");
        assert_eq!(PluginType::FileSystem.to_string(), "filesystem");
        assert_eq!(PluginType::Lister.to_string(), "lister");
        assert_eq!(PluginType::Native.to_string(), "native");
    }

    #[test]
    fn test_plugin_capabilities_default() {
        let caps = PluginCapabilities::default();
        assert!(!caps.can_create);
        assert!(!caps.can_modify);
        assert!(!caps.supports_encrypt);
    }

    #[test]
    fn test_archive_entry() {
        let entry = ArchiveEntry {
            path: "folder/file.txt".to_string(),
            size: 1024,
            compressed_size: Some(512),
            is_directory: false,
            modified_at: None,
        };
        assert!(!entry.is_directory);
        assert_eq!(entry.size, 1024);
    }
}
