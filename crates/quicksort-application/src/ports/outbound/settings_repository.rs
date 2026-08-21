//! Port for persisting and loading user settings.

use async_trait::async_trait;
use quicksort_domain::Settings;

use crate::errors::UseCaseError;

/// Repository for user settings persistence.
///
/// Implementations should store settings in a persistent medium
/// (e.g., JSON file, database). The Application Layer defines this
/// interface; Infrastructure provides the concrete implementation.
#[async_trait]
pub trait SettingsRepository: Send + Sync {
    /// Load settings from persistent storage.
    ///
    /// Returns default settings if no settings have been saved yet.
    async fn load(&self) -> Result<Settings, UseCaseError>;

    /// Save settings to persistent storage.
    async fn save(&self, settings: &Settings) -> Result<(), UseCaseError>;
}
