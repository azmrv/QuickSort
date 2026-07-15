//! Unified facade combining all inbound ports.

use std::sync::Arc;

use quicksort_domain::{Folder, FolderId, OperationId};

use crate::dtos::{OperationCommand, OperationResult};
use crate::errors::UseCaseError;
use crate::ports::inbound::{ExecuteOperation, GetFolders, ManageFolders, UndoOperation};
use crate::use_cases::{
    ExecuteOperationUseCase, UndoOperationUseCase, GetFoldersUseCase, ManageFoldersUseCase,
};

/// Unified facade combining all inbound operations.
pub struct ApplicationFacade {
    execute_operation: Arc<dyn ExecuteOperation>,
    undo_operation: Arc<dyn UndoOperation>,
    get_folders: Arc<dyn GetFolders>,
    manage_folders: Arc<dyn ManageFolders>,
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
        }
    }

    pub async fn execute_operation(&self, command: OperationCommand) -> Result<OperationResult, UseCaseError> {
        self.execute_operation.execute(command).await
    }

    pub async fn undo_operation(&self, operation_id: OperationId) -> Result<OperationResult, UseCaseError> {
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
}