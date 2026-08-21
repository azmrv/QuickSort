//! Inbound port for user settings operations.

use async_trait::async_trait;
use quicksort_domain::Settings;

use crate::errors::UseCaseError;

/// Trait for loading and saving user settings.
#[async_trait]
pub trait LoadSettings: Send + Sync {
    /// Load settings from persistent storage.
    async fn load_settings(&self) -> Result<Settings, UseCaseError>;
}

/// Trait for saving user settings.
#[async_trait]
pub trait SaveSettings: Send + Sync {
    /// Save settings to persistent storage.
    async fn save_settings(&self, settings: Settings) -> Result<(), UseCaseError>;
}
