//! Plugin Manager Use Case
//!
//! Manages plugin lifecycle: listing, enabling/disabling, configuration.
//! Delegates actual plugin loading to infrastructure layer.

use std::sync::Arc;

use async_trait::async_trait;
use quicksort_domain::PluginConfig;

use crate::errors::UseCaseError;
use crate::ports::inbound::{PluginInfoDto, PluginManager};

/// Port for plugin loading (implemented by infrastructure).
#[async_trait]
pub trait PluginLoader: Send + Sync {
    /// Discover and load all plugins.
    async fn discover_plugins(&self) -> Result<Vec<PluginInfoDto>, UseCaseError>;

    /// Get the directory where plugins are stored.
    fn plugin_directory(&self) -> &std::path::Path;
}

/// Port for plugin configuration persistence (implemented by infrastructure).
#[async_trait]
pub trait PluginConfigRepository: Send + Sync {
    /// Load plugin configuration.
    async fn load_config(&self, plugin_id: &str) -> Result<PluginConfig, UseCaseError>;

    /// Save plugin configuration.
    async fn save_config(&self, plugin_id: &str, config: &PluginConfig) -> Result<(), UseCaseError>;

    /// Check if plugin is enabled.
    async fn is_enabled(&self, plugin_id: &str) -> Result<bool, UseCaseError>;

    /// Set plugin enabled state.
    async fn set_enabled(&self, plugin_id: &str, enabled: bool) -> Result<(), UseCaseError>;
}

/// Concrete implementation of PluginManager use case.
pub struct PluginManagerUseCase {
    loader: Arc<dyn PluginLoader>,
    config_repo: Arc<dyn PluginConfigRepository>,
}

impl PluginManagerUseCase {
    pub fn new(
        loader: Arc<dyn PluginLoader>,
        config_repo: Arc<dyn PluginConfigRepository>,
    ) -> Self {
        Self {
            loader,
            config_repo,
        }
    }
}

#[async_trait]
impl PluginManager for PluginManagerUseCase {
    async fn list_plugins(&self) -> Result<Vec<PluginInfoDto>, UseCaseError> {
        let mut plugins = self.loader.discover_plugins().await?;

        // Enrich with enabled state from config
        for plugin in &mut plugins {
            match self.config_repo.is_enabled(&plugin.id).await {
                Ok(enabled) => plugin.enabled = enabled,
                Err(_) => plugin.enabled = true, // Default to enabled
            }
        }

        Ok(plugins)
    }

    async fn get_plugin_config(
        &self,
        plugin_id: &str,
    ) -> Result<PluginConfig, UseCaseError> {
        self.config_repo.load_config(plugin_id).await
    }

    async fn save_plugin_config(
        &self,
        plugin_id: &str,
        config: PluginConfig,
    ) -> Result<(), UseCaseError> {
        self.config_repo.save_config(plugin_id, &config).await
    }

    async fn set_plugin_enabled(
        &self,
        plugin_id: &str,
        enabled: bool,
    ) -> Result<(), UseCaseError> {
        self.config_repo.set_enabled(plugin_id, enabled).await
    }

    async fn rescan_plugins(&self) -> Result<Vec<PluginInfoDto>, UseCaseError> {
        self.list_plugins().await
    }
}
