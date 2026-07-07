//! Outbound port for configuration storage (folders, settings).

use async_trait::async_trait;
use quicksort_domain::{Folder, FolderId};
use crate::errors::UseCaseError;

#[async_trait]
pub trait ConfigurationRepository: Send + Sync {
    async fn load_all(&self) -> Result<Vec<Folder>, UseCaseError>;
    async fn save_all(&self, folders: &[Folder]) -> Result<(), UseCaseError>;
    async fn add(&self, folder: Folder) -> Result<(), UseCaseError>;
    async fn remove(&self, id: &FolderId) -> Result<(), UseCaseError>;
    async fn find_by_id(&self, id: &FolderId) -> Result<Option<Folder>, UseCaseError>;
    async fn find_by_path(&self, path: &str) -> Result<Option<Folder>, UseCaseError>;
}