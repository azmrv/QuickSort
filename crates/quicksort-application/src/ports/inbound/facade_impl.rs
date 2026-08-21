//! Application facade implementation combining all use cases.
//!
//! # Design Decision
//! The `ApplicationFacadeImpl` is a **single class** that implements all
//! inbound ports (`ExecuteOperation`, `UndoOperation`, `GetFolders`,
//! `ManageFolders`, `LoadSettings`, `SaveSettings`). This is intentional:
//! - Adapters (Tauri commands, IPC handlers) need only a single reference to
//!   the facade, not to individual use cases.
//! - It simplifies dependency injection and lifetime management.
//! - The facade itself contains **no business logic** — it merely delegates
//!   to the appropriate use case. This keeps the facade thin and testable.
//!
//! # Alternatives Considered
//! - **Separate classes per port**: Would require adapters to hold multiple
//!   references and duplicate wiring. Chosen approach reduces boilerplate.
//! - **Dynamic dispatch (trait objects)**: The facade could implement the
//!   ports as trait objects. However, using concrete types allows the
//!   compiler to inline calls, improving performance.

use async_trait::async_trait;
use std::sync::Arc;

use crate::dtos::{OperationCommand, OperationResult};
use crate::errors::UseCaseError;

use super::{
    ExecuteOperation, GetFolders, GetOperationHistory, LoadSettings, ManageFolders, SaveSettings,
    UndoOperation,
};
use crate::use_cases::{
    ExecuteOperationUseCase, GetFoldersUseCase, GetOperationHistoryUseCase, LoadSettingsUseCase,
    ManageFoldersUseCase, SaveSettingsUseCase, UndoOperationUseCase,
};
use quicksort_domain::{Folder, FolderId, Operation, OperationId, Settings};

/// Combined implementation of all inbound ports.
///
/// This struct serves as the **single entry point** for the entire
/// Application Layer. Every adapter (Tauri, CLI, IPC) calls methods
/// on this facade, which then delegates to the appropriate use case.
///
/// # Fields
/// - `execute` – Handles Move, Copy, Delete, Rename operations.
/// - `undo` – Reverts completed operations.
/// - `get_folders` – Retrieves the list of configured folders.
/// - `manage_folders` – CRUD operations for folder configuration.
/// - `load_settings` – Loads user settings.
/// - `save_settings` – Saves user settings.
pub struct ApplicationFacadeImpl {
    execute: Arc<ExecuteOperationUseCase>,
    undo: Arc<UndoOperationUseCase>,
    get_folders: Arc<GetFoldersUseCase>,
    manage_folders: Arc<ManageFoldersUseCase>,
    get_operation_history: Arc<GetOperationHistoryUseCase>,
    load_settings: Arc<LoadSettingsUseCase>,
    save_settings: Arc<SaveSettingsUseCase>,
}

impl ApplicationFacadeImpl {
    pub fn new(
        execute: Arc<ExecuteOperationUseCase>,
        undo: Arc<UndoOperationUseCase>,
        get_folders: Arc<GetFoldersUseCase>,
        manage_folders: Arc<ManageFoldersUseCase>,
        get_operation_history: Arc<GetOperationHistoryUseCase>,
        load_settings: Arc<LoadSettingsUseCase>,
        save_settings: Arc<SaveSettingsUseCase>,
    ) -> Self {
        Self {
            execute,
            undo,
            get_folders,
            manage_folders,
            get_operation_history,
            load_settings,
            save_settings,
        }
    }
}

// ---------------------------------------------------------------------------
// Inbound port implementations – pure delegation
// ---------------------------------------------------------------------------

#[async_trait]
impl ExecuteOperation for ApplicationFacadeImpl {
    /// Delegates to `ExecuteOperationUseCase::execute`.
    async fn execute(&self, command: OperationCommand) -> Result<OperationResult, UseCaseError> {
        self.execute.execute(command).await
    }
}

#[async_trait]
impl UndoOperation for ApplicationFacadeImpl {
    /// Delegates to `UndoOperationUseCase::undo`.
    async fn undo(&self, operation_id: OperationId) -> Result<OperationResult, UseCaseError> {
        self.undo.undo(operation_id).await
    }
}

#[async_trait]
impl GetFolders for ApplicationFacadeImpl {
    /// Delegates to `GetFoldersUseCase::get_all`.
    async fn get_all(&self) -> Result<Vec<Folder>, UseCaseError> {
        self.get_folders.get_all().await
    }
}

#[async_trait]
impl ManageFolders for ApplicationFacadeImpl {
    /// Delegates to `ManageFoldersUseCase::add_folder`.
    async fn add_folder(&self, folder: Folder) -> Result<(), UseCaseError> {
        self.manage_folders.add_folder(folder).await
    }

    /// Delegates to `ManageFoldersUseCase::remove_folder`.
    async fn remove_folder(&self, id: FolderId) -> Result<(), UseCaseError> {
        self.manage_folders.remove_folder(id).await
    }

    /// Delegates to `ManageFoldersUseCase::rename_folder`.
    async fn rename_folder(&self, id: FolderId, new_name: String) -> Result<(), UseCaseError> {
        self.manage_folders.rename_folder(id, new_name).await
    }

    /// Delegates to `ManageFoldersUseCase::toggle_favorite`.
    async fn toggle_favorite(&self, id: FolderId) -> Result<(), UseCaseError> {
        self.manage_folders.toggle_favorite(id).await
    }
}

#[async_trait]
impl GetOperationHistory for ApplicationFacadeImpl {
    /// Delegates to `GetOperationHistoryUseCase::get_all_operations`.
    async fn get_all_operations(&self) -> Result<Vec<Operation>, UseCaseError> {
        self.get_operation_history.get_all_operations().await
    }
}

#[async_trait]
impl LoadSettings for ApplicationFacadeImpl {
    /// Delegates to `LoadSettingsUseCase::load_settings`.
    async fn load_settings(&self) -> Result<Settings, UseCaseError> {
        self.load_settings.load_settings().await
    }
}

#[async_trait]
impl SaveSettings for ApplicationFacadeImpl {
    /// Delegates to `SaveSettingsUseCase::save_settings`.
    async fn save_settings(&self, settings: Settings) -> Result<(), UseCaseError> {
        self.save_settings.save_settings(settings).await
    }
}
