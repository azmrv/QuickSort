//! QuickSort Application Layer
//!
//! Provides ApplicationFacade as the unified entry point for all use cases,
//! managing the application lifecycle through Tauri.

use std::sync::Arc;
use quicksort_domain::{Folder, FolderId, OperationId};
use crate::dtos::{OperationCommand, OperationResult};
use crate::errors::UseCaseError;
use crate::ports::inbound::{ApplicationFacade as FacadeTrait, ExecuteOperation, GetFolders, ManageFolders, UndoOperation};
use crate::use_cases::{ExecuteOperationUseCase, GetFoldersUseCase, ManageFoldersUseCase, UndoOperationUseCase};

/// Facade pattern implementation for managing use cases.
pub struct ApplicationFacade {
    execute: Arc<dyn ExecuteOperation>,
    get_folders: Arc<dyn GetFolders>,
    manage: Arc<dyn ManageFolders>,
    undo: Arc<dyn UndoOperation>,
}

impl ApplicationFacade {
    /// Creates a new facade instance with the given use cases.
    pub fn new(
        execute: Arc<dyn ExecuteOperation>,
        get_folders: Arc<dyn GetFolders>,
        manage: Arc<dyn ManageFolders>,
        undo: Arc<dyn UndoOperation>,
    ) -> Self {
        Self {
            execute,
            get_folders,
            manage,
            undo,
        }
    }

    /// Executes a file operation (move, copy, delete, rename).
    pub async fn execute_operation(&self, command: OperationCommand) -> Result<OperationResult, UseCaseError> {
        self.execute.execute(command).await
    }

    /// Undoes an operation by its ID.
    pub async fn undo_operation(&self, operation_id: OperationId) -> Result<OperationResult, UseCaseError> {
        self.undo.undo(operation_id).await
    }

    /// Retrieves all saved folders.
    pub async fn get_folders(&self) -> Result<Vec<Folder>, UseCaseError> {
        self.get_folders.get_all().await
    }

    /// Adds a new folder.
    pub async fn add_folder(&self, folder: Folder) -> Result<(), UseCaseError> {
        self.manage.add_folder(folder).await
    }

    /// Removes a folder by ID.
    pub async fn remove_folder(&self, id: FolderId) -> Result<(), UseCaseError> {
        self.manage.remove_folder(id).await
    }

    /// Renames a folder.
    pub async fn rename_folder(&self, folder_id: FolderId, new_name: String) -> Result<(), UseCaseError> {
        self.manage.rename_folder(folder_id, new_name).await
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "tauri")))]
impl FacadeTrait for ApplicationFacade {
    async fn execute_operation(&self, command: OperationCommand) -> Result<OperationResult, UseCaseError> {
        self.execute_operation(command).await
    }

    async fn undo_operation(&self, id: OperationId) -> Result<OperationResult, UseCaseError> {
        self.undo_operation(id).await
    }

    async fn get_folders(&self) -> Result<Vec<Folder>, UseCaseError> {
        self.get_folders().await
    }

    async fn add_folder(&self, folder: Folder) -> Result<(), UseCaseError> {
        self.add_folder(folder).await
    }

    async fn remove_folder(&self, id: FolderId) -> Result<(), UseCaseError> {
        self.remove_folder(id).await
    }

    async fn rename_folder(&self, folder_id: FolderId, new_name: String) -> Result<(), UseCaseError> {
        self.rename_folder(folder_id, new_name).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::outbound::{
        ConfigurationRepository, OperationRepository, FileSystem, IdGenerator, Clock,
    };
    use std::sync::Arc;

    // Mock implementations for testing
    struct MockConfigRepo;
    struct MockOpRepo;
    struct MockFileSystem;
    struct MockIdGenerator;
    struct MockClock;

    #[tokio::test]
    async fn test_facade_creation() {
        let config_repo = Arc::new(MockConfigRepo);
        let op_repo = Arc::new(MockOpRepo);
        let fs = Arc::new(MockFileSystem);
        let id_gen = Arc::new(MockIdGenerator);
        let clock = Arc::new(MockClock);

        let execute = Arc::new(ExecuteOperationUseCase::new(
            config_repo.clone(),
            op_repo.clone(),
            fs.clone(),
            id_gen.clone(),
            clock.clone(),
        ));

        let get_folders = Arc::new(GetFoldersUseCase::new(config_repo.clone()));
        let manage = Arc::new(ManageFoldersUseCase::new(config_repo.clone()));
        let undo = Arc::new(UndoOperationUseCase::new(op_repo, fs, clock));

        let facade = ApplicationFacade::new(execute, get_folders, manage, undo);

        // This test just verifies the facade can be created.
        // Real tests should use proper mocks.
        assert!(facade.execute.is_some());
    }
}