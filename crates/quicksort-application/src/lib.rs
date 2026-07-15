//! QuickSort Application Layer
//!
//! Предоставляет фасад (ApplicationFacade) для взаимодействия с use case-ками
//! и управляет жизненным циклом приложения через Tauri.

use std::sync::Arc;
use quicksort_domain::{Folder, FolderId, OperationId};
use quicksort_application::dtos::{OperationCommand, OperationResult};
use quicksort_application::ports::inbound::{ApplicationFacade as FacadeTrait, ExecuteOperation, GetFolders, ManageFolders};
use quicksort_application::use_cases::{ExecuteOperationUseCase, GetFoldersUseCase, ManageFoldersUseCase};

/// Реализация фасадного паттерна для управления use case-ками
pub struct ApplicationFacade {
    execute: Arc<ExecuteOperationUseCase>,
    get_folders: Arc<GetFoldersUseCase>,
    manage: Arc<ManageFoldersUseCase>,
}

impl ApplicationFacade {
    /// Создает новый экземпляр фасадa с заданными use case-ками
    pub fn new(
        execute: ExecuteOperationUseCase,
        get_folders: GetFoldersUseCase,
        manage: ManageFoldersUseCase,
    ) -> Self {
        Self {
            execute: Arc::new(execute),
            get_folders: Arc::new(get_folders),
            manage: Arc::new(manage),
        }
    }

    /// Выполняет операцию (move, copy, delete, rename)
    pub async fn execute_operation(&self, command: OperationCommand) -> Result<OperationResult, crate::errors::UseCaseError> {
        ExecuteOperation::execute(&*self.execute, command).await
    }

    /// Откатывает операцию по её идентификатору
    pub async fn undo_operation(&self, operation_id: OperationId) -> Result<OperationResult, crate::errors::UseCaseError> {
        crate::use_cases::UndoOperationUseCase::execute(&operation_id).await
    }

    /// Получает все сохранённые папки
    pub async fn get_folders(&self) -> Result<Vec<Folder>, crate::errors::UseCaseError> {
        GetFolders::get_all(&*self.get_folders).await
    }

    /// Добавляет новую папку
    pub async fn add_folder(&self, folder: Folder) -> Result<(), crate::errors::UseCaseError> {
        ManageFolders::add_folder(&*self.manage, folder).await
    }

    /// Удаляет папку по идентификатору
    pub async fn remove_folder(&self, id: FolderId) -> Result<(), crate::errors::UseCaseError> {
        ManageFolders::remove_folder(&*self.manage, id).await
    }

    /// Переименовывает папку
    pub async fn rename_folder(
        &self,
        folder_id: FolderId,
        new_name: String,
    ) -> Result<(), crate::errors::UseCaseError> {
        use quicksort_application::ports::inbound::RenameFolder;
        RenameFolder::rename_folder(&*self.manage, folder_id, new_name).await
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "tauri")))]
impl FacadeTrait for ApplicationFacade {
    async fn execute_operation(&self, command: OperationCommand) -> Result<OperationResult, crate::errors::UseCaseError> {
        self.execute_operation(command).await
    }

    async fn undo_operation(&self, id: OperationId) -> Result<OperationResult, crate::errors::UseCaseError> {
        self.undo_operation(id).await
    }

    async fn get_folders(&self) -> Result<Vec<Folder>, crate::errors::UseCaseError> {
        self.get_folders().await
    }

    async fn add_folder(&self, folder: Folder) -> Result<(), crate::errors::UseCaseError> {
        self.add_folder(folder).await
    }

    async fn remove_folder(&self, id: FolderId) -> Result<(), crate::errors::UseCaseError> {
        self.remove_folder(id).await
    }

    async fn rename_folder(
        &self,
        folder_id: FolderId,
        new_name: String,
    ) -> Result<(), crate::errors::UseCaseError> {
        self.rename_folder(folder_id, new_name).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quicksort_domain::WindowsPath;

    #[tokio::test]
    async fn test_facade_creation() {
        let facade = ApplicationFacade::new(
            ExecuteOperationUseCase::default(),
            GetFoldersUseCase::default(),
            ManageFoldersUseCase::default(),
        );
        assert!(facade.execute_operation(OperationCommand::Move {}).await.is_err());
    }

    #[tokio::test]
    async fn test_facade_get_folders() {
        let facade = ApplicationFacade::new(
            ExecuteOperationUseCase::default(),
            GetFoldersUseCase::default(),
            ManageFoldersUseCase::default(),
        );
        let folders = facade.get_folders().await;
        assert!(folders.is_err()); // Ожидаем ошибку, так как репозиторий не настроен
    }
}