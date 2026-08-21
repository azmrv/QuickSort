//! Unified facade combining all inbound ports.

use std::sync::Arc;

use quicksort_domain::{Folder, FolderId, Operation, OperationId, PluginConfig, Settings};

use crate::dtos::{OperationCommand, OperationResult};
use crate::errors::UseCaseError;
use crate::ports::inbound::{
    ExecuteOperation, GetFolders, GetOperationHistory, ManageFolders, PluginInfoDto, PluginManager,
    UndoOperation,
};

/// Unified facade combining all inbound operations.
pub struct ApplicationFacade {
    execute_operation: Arc<dyn ExecuteOperation>,
    undo_operation: Arc<dyn UndoOperation>,
    get_folders: Arc<dyn GetFolders>,
    manage_folders: Arc<dyn ManageFolders>,
    get_operation_history: Arc<dyn GetOperationHistory>,
    load_settings: Option<Arc<dyn crate::ports::inbound::LoadSettings>>,
    save_settings: Option<Arc<dyn crate::ports::inbound::SaveSettings>>,
    plugin_manager: Option<Arc<dyn PluginManager>>,
}

impl ApplicationFacade {
    pub fn new(
        execute_operation: Arc<dyn ExecuteOperation>,
        undo_operation: Arc<dyn UndoOperation>,
        get_folders: Arc<dyn GetFolders>,
        manage_folders: Arc<dyn ManageFolders>,
        get_operation_history: Arc<dyn GetOperationHistory>,
    ) -> Self {
        Self {
            execute_operation,
            undo_operation,
            get_folders,
            manage_folders,
            get_operation_history,
            load_settings: None,
            save_settings: None,
            plugin_manager: None,
        }
    }

    pub fn with_settings(
        mut self,
        load_settings: Arc<dyn crate::ports::inbound::LoadSettings>,
        save_settings: Arc<dyn crate::ports::inbound::SaveSettings>,
    ) -> Self {
        self.load_settings = Some(load_settings);
        self.save_settings = Some(save_settings);
        self
    }

    pub fn with_plugin_manager(
        mut self,
        plugin_manager: Arc<dyn PluginManager>,
    ) -> Self {
        self.plugin_manager = Some(plugin_manager);
        self
    }

    pub async fn execute_operation(
        &self,
        command: OperationCommand,
    ) -> Result<OperationResult, UseCaseError> {
        self.execute_operation.execute(command).await
    }

    pub async fn undo_operation(
        &self,
        operation_id: OperationId,
    ) -> Result<OperationResult, UseCaseError> {
        self.undo_operation.undo(operation_id).await
    }

    pub async fn get_folders(&self) -> Result<Vec<Folder>, UseCaseError> {
        self.get_folders.get_all().await
    }

    pub async fn add_folder(&self, folder: Folder) -> Result<(), UseCaseError> {
        self.manage_folders.add_folder(folder).await
    }

    pub async fn remove_folder(&self, id: FolderId) -> Result<(), UseCaseError> {
        self.manage_folders.remove_folder(id).await
    }

    pub async fn rename_folder(&self, id: FolderId, new_name: String) -> Result<(), UseCaseError> {
        self.manage_folders.rename_folder(id, new_name).await
    }

    pub async fn get_operation_history(&self) -> Result<Vec<Operation>, UseCaseError> {
        self.get_operation_history.get_all_operations().await
    }

    pub async fn load_settings(&self) -> Result<Settings, UseCaseError> {
        self.load_settings
            .as_ref()
            .ok_or(UseCaseError::RepositoryError(
                "Settings not configured".to_string(),
            ))?
            .load_settings()
            .await
    }

    pub async fn save_settings(&self, settings: Settings) -> Result<(), UseCaseError> {
        self.save_settings
            .as_ref()
            .ok_or(UseCaseError::RepositoryError(
                "Settings not configured".to_string(),
            ))?
            .save_settings(settings)
            .await
    }

    pub async fn list_plugins(&self) -> Result<Vec<PluginInfoDto>, UseCaseError> {
        self.plugin_manager
            .as_ref()
            .ok_or(UseCaseError::RepositoryError(
                "Plugin manager not configured".to_string(),
            ))?
            .list_plugins()
            .await
    }

    pub async fn get_plugin_config(
        &self,
        plugin_id: &str,
    ) -> Result<PluginConfig, UseCaseError> {
        self.plugin_manager
            .as_ref()
            .ok_or(UseCaseError::RepositoryError(
                "Plugin manager not configured".to_string(),
            ))?
            .get_plugin_config(plugin_id)
            .await
    }

    pub async fn save_plugin_config(
        &self,
        plugin_id: &str,
        config: PluginConfig,
    ) -> Result<(), UseCaseError> {
        self.plugin_manager
            .as_ref()
            .ok_or(UseCaseError::RepositoryError(
                "Plugin manager not configured".to_string(),
            ))?
            .save_plugin_config(plugin_id, config)
            .await
    }

    pub async fn set_plugin_enabled(
        &self,
        plugin_id: &str,
        enabled: bool,
    ) -> Result<(), UseCaseError> {
        self.plugin_manager
            .as_ref()
            .ok_or(UseCaseError::RepositoryError(
                "Plugin manager not configured".to_string(),
            ))?
            .set_plugin_enabled(plugin_id, enabled)
            .await
    }

    pub async fn rescan_plugins(&self) -> Result<Vec<PluginInfoDto>, UseCaseError> {
        self.plugin_manager
            .as_ref()
            .ok_or(UseCaseError::RepositoryError(
                "Plugin manager not configured".to_string(),
            ))?
            .rescan_plugins()
            .await
    }
}
