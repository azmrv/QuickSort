//! GetFoldersUseCase - returns all configured folders.

use async_trait::async_trait;
use std::sync::Arc;
use quicksort_domain::Folder;
use crate::errors::UseCaseError;
use crate::ports::outbound::ConfigurationRepository;
use crate::ports::inbound::GetFolders;

pub struct GetFoldersUseCase {
    config_repo: Arc<dyn ConfigurationRepository>,
}

impl GetFoldersUseCase {
    pub fn new(config_repo: Arc<dyn ConfigurationRepository>) -> Self {
        Self { config_repo }
    }
}

#[async_trait]
impl GetFolders for GetFoldersUseCase {
    async fn get_all(&self) -> Result<Vec<Folder>, UseCaseError> {
        self.config_repo.load_all().await
    }
}