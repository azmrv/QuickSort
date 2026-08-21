//! JSON-based implementation of SettingsRepository.
//! Stores user settings in a JSON file.

use async_trait::async_trait;
use quicksort_application::errors::UseCaseError;
use quicksort_application::ports::outbound::SettingsRepository;
use quicksort_domain::Settings;
use std::fs;
use std::path::PathBuf;

/// Repository that stores user settings in a JSON file.
pub struct JsonSettingsRepository {
    path: PathBuf,
}

impl JsonSettingsRepository {
    /// Creates a new repository with the given file path.
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    /// Load settings from file, returning defaults if file doesn't exist.
    fn load_from_file(&self) -> Result<Settings, UseCaseError> {
        if !self.path.exists() {
            return Ok(Settings::default());
        }
        let content = fs::read_to_string(&self.path)
            .map_err(|e| UseCaseError::RepositoryError(e.to_string()))?;
        let settings: Settings = serde_json::from_str(&content)
            .map_err(|e| UseCaseError::RepositoryError(e.to_string()))?;
        Ok(settings)
    }

    /// Save settings to file.
    fn save_to_file(&self, settings: &Settings) -> Result<(), UseCaseError> {
        let content = serde_json::to_string_pretty(settings)
            .map_err(|e| UseCaseError::RepositoryError(e.to_string()))?;
        fs::write(&self.path, content).map_err(|e| UseCaseError::RepositoryError(e.to_string()))?;
        Ok(())
    }
}

#[async_trait]
impl SettingsRepository for JsonSettingsRepository {
    async fn load(&self) -> Result<Settings, UseCaseError> {
        self.load_from_file()
    }

    async fn save(&self, settings: &Settings) -> Result<(), UseCaseError> {
        // Ensure parent directory exists
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|e| UseCaseError::RepositoryError(e.to_string()))?;
        }
        self.save_to_file(settings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_load_default_when_no_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("settings.json");
        let repo = JsonSettingsRepository::new(path);

        let settings = repo.load().await.unwrap();
        assert_eq!(settings, Settings::default());
    }

    #[tokio::test]
    async fn test_save_and_load() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("settings.json");
        let repo = JsonSettingsRepository::new(path);

        let mut settings = Settings::default();
        settings.default_operation = quicksort_domain::DefaultOperation::Copy;

        repo.save(&settings).await.unwrap();
        let loaded = repo.load().await.unwrap();

        assert_eq!(
            loaded.default_operation,
            quicksort_domain::DefaultOperation::Copy
        );
    }

    #[tokio::test]
    async fn test_creates_parent_directory() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("subdir").join("settings.json");
        let repo = JsonSettingsRepository::new(path.clone());

        let settings = Settings::default();
        repo.save(&settings).await.unwrap();

        assert!(path.exists());
    }
}
