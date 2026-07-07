//! Port for managing folders (add, remove, rename, reorder).

use async_trait::async_trait;
use crate::errors::UseCaseError;
use quicksort_domain::{Folder, FolderId};

#[async_trait]
pub trait ManageFolders: Send + Sync {
    async fn add_folder(&self, folder: Folder) -> Result<(), UseCaseError>;
    async fn remove_folder(&self, id: FolderId) -> Result<(), UseCaseError>;
    async fn rename_folder(&self, id: FolderId, new_name: String) -> Result<(), UseCaseError>;
    async fn toggle_favorite(&self, id: FolderId, order: i32) -> Result<(), UseCaseError>;
}