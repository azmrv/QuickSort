//! Plugin management port.

use async_trait::async_trait;
use quicksort_domain::{PluginConfig, PluginType};

/// Plugin information for the UI.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PluginInfoDto {
    pub id: String,
    pub name: String,
    pub version: String,
    pub plugin_type: PluginType,
    pub enabled: bool,
    pub path: String,
}

/// Port for listing and managing plugins.
#[async_trait]
pub trait PluginManager: Send + Sync {
    /// List all discovered plugins.
    async fn list_plugins(&self) -> Result<Vec<PluginInfoDto>, crate::errors::UseCaseError>;

    /// Get plugin configuration.
    async fn get_plugin_config(
        &self,
        plugin_id: &str,
    ) -> Result<PluginConfig, crate::errors::UseCaseError>;

    /// Save plugin configuration.
    async fn save_plugin_config(
        &self,
        plugin_id: &str,
        config: PluginConfig,
    ) -> Result<(), crate::errors::UseCaseError>;

    /// Enable or disable a plugin.
    async fn set_plugin_enabled(
        &self,
        plugin_id: &str,
        enabled: bool,
    ) -> Result<(), crate::errors::UseCaseError>;

    /// Rescan plugin directory.
    async fn rescan_plugins(&self) -> Result<Vec<PluginInfoDto>, crate::errors::UseCaseError>;
}
