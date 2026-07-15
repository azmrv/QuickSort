//! GetFoldersUseCase - returns all configured folders.
//!
//! # Responsibility
//! Simply loads the entire folder list from the configuration repository.
//! No business logic beyond that – filtering, sorting, or mapping is kept
//! in the domain layer or in other use cases.
//!
//! # Dependencies
//! - `ConfigurationRepository` (outbound port) – provides access to stored folders.

use async_trait::async_trait;
use std::sync::Arc;
use quicksort_domain::Folder;
use crate::errors::UseCaseError;
use crate::ports::outbound::ConfigurationRepository;
use crate::ports::inbound::GetFolders;

/// A straightforward use case that retrieves all configured folders.
///
/// It performs no filtering or transformation; it merely acts as a thin
/// abstraction over the repository to satisfy the inbound port contract.
pub struct GetFoldersUseCase {
    config_repo: Arc<dyn ConfigurationRepository>,
}

impl GetFoldersUseCase {
    /// Constructs a new instance with the given configuration repository.
    pub fn new(config_repo: Arc<dyn ConfigurationRepository>) -> Self {
        Self { config_repo }
    }
}

#[async_trait]
impl GetFolders for GetFoldersUseCase {
    /// Returns all folders stored in the configuration repository.
    ///
    /// # Errors
    /// Returns `UseCaseError::RepositoryError` if the underlying storage
    /// fails (e.g., file not found, malformed JSON, I/O error).
    async fn get_all(&self) -> Result<Vec<Folder>, UseCaseError> {
        // Delegate to the repository, mapping any infrastructure error
        // into our application-level error type.
        self.config_repo
            .load_all()
            .await
            .map_err(|e| UseCaseError::RepositoryError(e.to_string()))
    }
}