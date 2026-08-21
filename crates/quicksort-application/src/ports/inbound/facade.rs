//! Unified facade combining all inbound ports.

use std::sync::Arc;

use quicksort_domain::{Folder, FolderId, OperationId, Settings};

use crate::dtos::{OperationCommand, OperationResult};
use crate::errors::UseCaseError;
use crate::ports::inbound::{ExecuteOperation, GetFolders, ManageFolders, UndoOperation};

/// Unified facade combining all inbound operations.
pub struct ApplicationFacade {
    execute_operation: Arc<dyn ExecuteOperation>,
    undo_operation: Arc<dyn UndoOperation>,
    get_folders: Arc<dyn GetFolders>,
    manage_folders: Arc<dyn ManageFolders>,
    load_settings: Option<Arc<dyn crate::ports::inbound::LoadSettings>>,
    save_settings: Option<Arc<dyn crate::ports::inbound::SaveSettings>>,
}

impl ApplicationFacade {
    pub fn new(
        execute_operation: Arc<dyn ExecuteOperation>,
        undo_operation: Arc<dyn UndoOperation>,
        get_folders: Arc<dyn GetFolders>,
        manage_folders: Arc<dyn ManageFolders>,
    ) -> Self {
        Self {
            execute_operation,
            undo_operation,
            get_folders,
            manage_folders,
            load_settings: None,
            save_settings: None,
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
}
