//! Domain entity for user settings and preferences.

use serde::{Deserialize, Serialize};

/// Default operation type when executing file operations.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum DefaultOperation {
    #[default]
    Move,
    Copy,
}

/// Default action when a duplicate file is found.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum DefaultOverwritePolicy {
    Skip,
    Overwrite,
    #[default]
    AutoRename,
}

/// Duplicate detection mode.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DuplicateCheckMode {
    /// Quick check: file with same name exists at destination.
    Name,
    /// Medium check: same name AND same file size.
    #[default]
    Size,
    /// Deep check: SHA-256 hash comparison (slowest, most accurate).
    Content,
}

/// Duplicate detection configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DuplicateCheckConfig {
    /// Whether duplicate checking is enabled.
    pub enabled: bool,
    /// Detection mode.
    pub mode: DuplicateCheckMode,
}

impl Default for DuplicateCheckConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            mode: DuplicateCheckMode::default(),
        }
    }
}

/// Theme mode selection.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    /// Follow OS system theme.
    #[default]
    System,
    /// Always light theme.
    Light,
    /// Always dark theme.
    Dark,
}

/// UI language locale identifier.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Locale {
    #[default]
    En,
    Ru,
    De,
    Es,
    Zh,
    Ja,
}

/// Logging verbosity level.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    /// Trace — most verbose, includes detailed debug tracing.
    Trace,
    /// Debug — extended debug information.
    Debug,
    /// Info — normal operational events.
    #[default]
    Info,
    /// Warn — suspicious or recoverable conditions.
    Warn,
    /// Error — failures and unrecoverable conditions.
    Error,
}

/// Log output format.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    /// Human-readable text lines.
    #[default]
    Text,
    /// Structured JSON lines (applies to the file log only).
    Json,
}

/// Logging configuration.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Verbosity level applied when no `RUST_LOG` override is set.
    pub level: LogLevel,
    /// Output format for the file log. The stdout and frontend streams keep
    /// their own fixed formats regardless of this value.
    pub format: LogFormat,
    /// Optional log file name override. The parent directory selects the log
    /// folder and the file stem becomes the rotated file prefix
    /// (e.g. `custom` -> `custom.2026-09-09.log`). When `None`, the default
    /// `quicksort` prefix is used.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
    /// Optional limit on how many rotated log files are kept on disk.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_files: Option<u32>,
}

/// User settings entity.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Settings {
    /// Default operation type (Move or Copy).
    #[serde(default)]
    pub default_operation: DefaultOperation,
    /// Default overwrite policy when duplicate found.
    #[serde(default)]
    pub default_overwrite_policy: DefaultOverwritePolicy,
    /// Duplicate detection configuration.
    #[serde(default)]
    pub duplicate_check: DuplicateCheckConfig,
    /// Theme mode: system, light, or dark.
    #[serde(default)]
    pub theme_mode: ThemeMode,
    /// UI language.
    #[serde(default)]
    pub locale: Locale,
    /// Logging configuration.
    #[serde(default)]
    pub logging: LoggingConfig,
}

impl Settings {
    /// Creates new settings with default values.
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = Settings::default();
        assert_eq!(settings.default_operation, DefaultOperation::Move);
        assert_eq!(
            settings.default_overwrite_policy,
            DefaultOverwritePolicy::AutoRename
        );
        assert!(settings.duplicate_check.enabled);
        assert_eq!(settings.duplicate_check.mode, DuplicateCheckMode::Size);
        assert_eq!(settings.theme_mode, ThemeMode::System);
        assert_eq!(settings.locale, Locale::En);
        assert_eq!(settings.logging.level, LogLevel::Info);
        assert_eq!(settings.logging.format, LogFormat::Text);
        assert!(settings.logging.file_path.is_none());
        assert!(settings.logging.max_files.is_none());
    }

    #[test]
    fn test_serialize_deserialize() {
        let settings = Settings::default();
        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: Settings = serde_json::from_str(&json).unwrap();
        assert_eq!(settings, deserialized);
    }

    #[test]
    fn test_json_format() {
        let settings = Settings::default();
        let json = serde_json::to_string_pretty(&settings).unwrap();
        assert!(json.contains("\"Move\""));
        assert!(json.contains("\"AutoRename\""));
        assert!(json.contains("\"size\""));
        assert!(json.contains("\"system\""));
        assert!(json.contains("\"en\""));
        assert!(json.contains("\"logging\""));
        assert!(json.contains("\"info\""));
        assert!(json.contains("\"text\""));
    }

    #[test]
    fn test_backward_compat_missing_fields() {
        // Old settings.json without theme_mode or locale should deserialize fine.
        let old_json = r#"{"default_operation":"Move","default_overwrite_policy":"AutoRename","duplicate_check":{"enabled":true,"mode":"size"}}"#;
        let settings: Settings = serde_json::from_str(old_json).unwrap();
        assert_eq!(settings.theme_mode, ThemeMode::System);
        assert_eq!(settings.locale, Locale::En);
        assert_eq!(settings.logging, LoggingConfig::default());
    }

    #[test]
    fn test_logging_config_round_trip() {
        let config = LoggingConfig {
            level: LogLevel::Debug,
            format: LogFormat::Json,
            file_path: Some("debug.log".into()),
            max_files: Some(7),
        };
        let settings = Settings {
            logging: config.clone(),
            ..Settings::default()
        };
        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: Settings = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.logging, config);
    }

    #[test]
    fn test_logging_config_defaults_omit_optional_fields() {
        let json = serde_json::to_string(&LoggingConfig::default()).unwrap();
        assert!(!json.contains("file_path"));
        assert!(!json.contains("max_files"));
    }
}
