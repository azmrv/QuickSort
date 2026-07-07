//! Port for retrieving folders from configuration.

use async_trait::async_trait;
use crate::errors::UseCaseError;
use quicksort_domain::Folder;

#[async_trait]
pub trait GetFolders: Send + Sync {
    /// Returns all configured folders.
    async fn get_all(&self) -> Result<Vec<Folder>, UseCaseError>;
}