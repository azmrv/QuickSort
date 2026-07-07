//! Application facade implementation combining all use cases.

use std::sync::Arc;
use async_trait::async_trait;

use crate::dtos::{OperationCommand, OperationResult};
use crate::errors::UseCaseError;
use crate::use_cases::{
    ExecuteOperationUseCase, UndoOperationUseCase,
    GetFoldersUseCase, ManageFoldersUseCase,
};
use super::{ExecuteOperation, UndoOperation, GetFolders, ManageFolders};
use quicksort_domain::{Folder, FolderId, OperationId};

/// Combined implementation of all inbound ports.
pub struct ApplicationFacadeImpl {
    pub execute: Arc<ExecuteOperationUseCase>,
    pub undo: Arc<UndoOperationUseCase>,
    pub get_folders: Arc<GetFoldersUseCase>,
    pub manage_folders: Arc<ManageFoldersUseCase>,
}

#[async_trait]
impl ExecuteOperation for ApplicationFacadeImpl {
    async fn execute(&self, command: OperationCommand) -> Result<OperationResult, UseCaseError> {
        self.execute.execute(command).await
    }
}

#[async_trait]
impl UndoOperation for ApplicationFacadeImpl {
    async fn undo(&self, operation_id: OperationId) -> Result<OperationResult, UseCaseError> {
        self.undo.undo(operation_id).await
    }
}

#[async_trait]
impl GetFolders for ApplicationFacadeImpl {
    async fn get_all(&self) -> Result<Vec<Folder>, UseCaseError> {
        self.get_folders.get_all().await
    }
}

#[async_trait]
impl ManageFolders for ApplicationFacadeImpl {
    async fn add_folder(&self, folder: Folder) -> Result<(), UseCaseError> {
        self.manage_folders.add_folder(folder).await
    }

    async fn remove_folder(&self, id: FolderId) -> Result<(), UseCaseError> {
        self.manage_folders.remove_folder(id).await
    }

    async fn rename_folder(&self, id: FolderId, new_name: String) -> Result<(), UseCaseError> {
        self.manage_folders.rename_folder(id, new_name).await
    }

    async fn toggle_favorite(&self, id: FolderId, order: i32) -> Result<(), UseCaseError> {
        self.manage_folders.toggle_favorite(id, order).await
    }
}