//! Inbound port for retrieving folders from configuration.
//!
//! This port is part of the Application Layer's inbound interface.
//! It is implemented by `GetFoldersUseCase` and called by adapters
//! (Tauri commands, IPC handlers) to obtain the list of configured folders.

use async_trait::async_trait;
use crate::errors::UseCaseError;
use quicksort_domain::Folder;

/// Inbound port for reading the folder configuration.
///
/// # Role in the Architecture
/// - **Inbound port** – called by outer layers (adapters).
/// - **Implemented by** – `GetFoldersUseCase` (Application Layer).
/// - **Used by** – Tauri commands, IPC server, any UI that needs the folder list.
///
/// # Errors
/// Returns `UseCaseError::RepositoryError` if the underlying storage
/// cannot be read (e.g., missing file, malformed JSON, I/O failure).
#[async_trait]
pub trait GetFolders: Send + Sync {
    /// Returns all folders currently stored in the configuration.
    ///
    /// The returned list is unfiltered and unsorted; any ordering or filtering
    /// is the responsibility of the caller (or the UI layer).
    async fn get_all(&self) -> Result<Vec<Folder>, UseCaseError>;
}