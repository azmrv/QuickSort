//! ManageFoldersUseCase - add, remove, rename, reorder folders.

use async_trait::async_trait;
use std::sync::Arc;
use quicksort_domain::{Folder, FolderId};
use crate::errors::UseCaseError;
use crate::ports::outbound::ConfigurationRepository;
use crate::ports::inbound::ManageFolders;

pub struct ManageFoldersUseCase {
    config_repo: Arc<dyn ConfigurationRepository>,
}

impl ManageFoldersUseCase {
    pub fn new(config_repo: Arc<dyn ConfigurationRepository>) -> Self {
        Self { config_repo }
    }
}

#[async_trait]
impl ManageFolders for ManageFoldersUseCase {
    async fn add_folder(&self, folder: Folder) -> Result<(), UseCaseError> {
        self.config_repo.add(folder).await
    }

    async fn remove_folder(&self, id: FolderId) -> Result<(), UseCaseError> {
        self.config_repo.remove(&id).await
    }

    async fn rename_folder(&self, id: FolderId, new_name: String) -> Result<(), UseCaseError> {
        let mut folders = self.config_repo.load_all().await?;
        let folder = folders.iter_mut().find(|f| f.id == id)
            .ok_or_else(|| UseCaseError::FolderNotFound(id.as_str().to_string()))?;
        folder.name = new_name;
        self.config_repo.save_all(&folders).await
    }

    async fn toggle_favorite(&self, id: FolderId, order: i32) -> Result<(), UseCaseError> {
        let mut folders = self.config_repo.load_all().await?;
        let folder = folders.iter_mut().find(|f| f.id == id)
            .ok_or_else(|| UseCaseError::FolderNotFound(id.as_str().to_string()))?;
        folder.is_favorite = !folder.is_favorite;
        folder.sort_order = if folder.is_favorite { order } else { 0 };
        self.config_repo.save_all(&folders).await
    }
}