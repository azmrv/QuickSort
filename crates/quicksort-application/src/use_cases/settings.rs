//! Use cases for loading and saving user settings.

use std::sync::Arc;

use async_trait::async_trait;
use quicksort_domain::Settings;

use crate::errors::UseCaseError;
use crate::ports::inbound::{LoadSettings, SaveSettings};
use crate::ports::outbound::SettingsRepository;

/// Use case for loading user settings.
pub struct LoadSettingsUseCase {
    repository: Arc<dyn SettingsRepository>,
}

impl LoadSettingsUseCase {
    pub fn new(repository: Arc<dyn SettingsRepository>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl LoadSettings for LoadSettingsUseCase {
    async fn load_settings(&self) -> Result<Settings, UseCaseError> {
        self.repository.load().await
    }
}

/// Use case for saving user settings.
pub struct SaveSettingsUseCase {
    repository: Arc<dyn SettingsRepository>,
}

impl SaveSettingsUseCase {
    pub fn new(repository: Arc<dyn SettingsRepository>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl SaveSettings for SaveSettingsUseCase {
    async fn save_settings(&self, settings: Settings) -> Result<(), UseCaseError> {
        self.repository.save(&settings).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct MockSettingsRepository {
        settings: Mutex<Settings>,
    }

    impl MockSettingsRepository {
        fn new() -> Self {
            Self {
                settings: Mutex::new(Settings::default()),
            }
        }
    }

    #[async_trait]
    impl SettingsRepository for MockSettingsRepository {
        async fn load(&self) -> Result<Settings, UseCaseError> {
            Ok(self.settings.lock().unwrap().clone())
        }

        async fn save(&self, settings: &Settings) -> Result<(), UseCaseError> {
            *self.settings.lock().unwrap() = settings.clone();
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_load_settings() {
        let repo = Arc::new(MockSettingsRepository::new());
        let use_case = LoadSettingsUseCase::new(repo);
        let settings = use_case.load_settings().await.unwrap();
        assert_eq!(
            settings.default_operation,
            quicksort_domain::DefaultOperation::Move
        );
    }

    #[tokio::test]
    async fn test_save_settings() {
        let repo = Arc::new(MockSettingsRepository::new());
        let use_case = SaveSettingsUseCase::new(repo.clone());

        let mut settings = Settings::default();
        settings.default_operation = quicksort_domain::DefaultOperation::Copy;

        use_case.save_settings(settings).await.unwrap();

        let load_use_case = LoadSettingsUseCase::new(repo);
        let loaded = load_use_case.load_settings().await.unwrap();
        assert_eq!(
            loaded.default_operation,
            quicksort_domain::DefaultOperation::Copy
        );
    }
}
