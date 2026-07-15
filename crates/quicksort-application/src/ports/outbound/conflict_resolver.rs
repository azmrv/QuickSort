//! Outbound port for conflict resolution.
//!
//! This port defines the interface for resolving file conflicts that occur
//! during file operations (e.g., when a destination file already exists and
//! the chosen policy is `Ask`).
//!
//! # Current Status
//! **Not yet used in production.** The current implementation of
//! `ExecuteOperationUseCase::resolve_destination` handles `Skip`, `Overwrite`,
//! and `AutoRename` policies internally. The `Ask` policy falls back to
//! `AutoRename` in non-interactive contexts.
//!
//! This port is reserved for a future enhancement where the adapter layer
//! (e.g., Tauri GUI) can present a dialog to the user and invoke this port
//! to apply the user's choice (Overwrite, Skip, Rename).

use async_trait::async_trait;
use crate::dtos::OperationCommand;
use crate::errors::UseCaseError;

/// Port for resolving file conflicts interactively.
///
/// # Planned Usage (Future)
/// When a file operation encounters a conflict (e.g., `OverwritePolicy::Ask`),
/// the Use Case will call this port with the command that caused the conflict.
/// The infrastructure adapter (e.g., a GUI dialog) will then prompt the user
/// and return a modified `OperationCommand` with the resolved policy (e.g.,
/// `Overwrite`, `Skip`, or a user-specified new name).
///
/// # Current Behavior
/// This port is **not yet called** by any Use Case. The `Ask` policy is
/// currently handled by falling back to `AutoRename`. See
/// `ExecuteOperationUseCase::resolve_destination` for details.
#[async_trait]
pub trait ConflictResolver: Send + Sync {
    /// Resolves a file conflict by modifying the operation command.
    ///
    /// # Arguments
    /// * `command` – The command that caused the conflict. The resolver may
    ///   modify fields such as `overwrite_policy` or `target_paths` to
    ///   reflect the user's decision.
    ///
    /// # Returns
    /// A potentially modified `OperationCommand` with the conflict resolved,
    /// or an error if the user cancels the operation.
    ///
    /// # Errors
    /// Returns `UseCaseError::Conflict` if the user chooses to abort the operation.
    async fn resolve(&self, command: OperationCommand) -> Result<OperationCommand, UseCaseError>;
}