//! Application-level error types.
//!
//! All Use Cases return errors of this type.
//! Adapters map these to user-friendly messages.
//!
//! # Design Decisions
//! - Each variant clearly indicates the source of the error (domain,
//!   repository, file system), making it easy to route errors to the
//!   appropriate handler (e.g., user notification vs. log-and-retry).
//! - The `Domain` variant wraps domain-level errors, preserving the
//!   original context for debugging while keeping the API uniform.
//! - The `Internal` variant is reserved for truly unexpected situations
//!   (e.g., poisoned locks, broken invariants that should never occur).
//!
//! # Error Handling Strategy
//! - **Domain errors** (`FolderNotFound`, `FileNotFound`, etc.) are
//!   triggered by business rule violations and are typically shown to
//!   the user as informational messages.
//! - **Infrastructure errors** (`RepositoryError`, `FileSystemError`)
//!   are caused by external systems (disk, database, network). Adapters
//!   should log these and may retry or escalate depending on severity.
//! - **Internal errors** (`Internal`) signal programmer mistakes or
//!   unrecoverable states and should be logged with high priority.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Unified error type for the Application layer.
///
/// All public methods in the Use Cases return `Result<T, UseCaseError>`.
/// Adapters (Tauri commands, IPC handlers) map these to appropriate
/// responses (e.g., HTTP status codes, user notifications, log entries).
///
/// # Example
/// ```rust,ignore
/// use quicksort_application::UseCaseError;
/// fn handle_command() -> Result<(), UseCaseError> {
///     // Trigger a domain error
///     Err(UseCaseError::FolderNotFound("MyFolder".to_string()))
/// }
/// ```
#[derive(Debug, Error)]
pub enum UseCaseError {
    // ---- Domain-level errors ----
    // These errors originate from business rule violations.
    /// The requested folder does not exist or is inaccessible.
    #[error("Folder not found: {0}")]
    FolderNotFound(String),

    /// The requested file does not exist or is inaccessible.
    #[error("File not found: {0}")]
    FileNotFound(String),

    /// The operation cannot be performed due to insufficient permissions.
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// A conflict occurred (e.g., file already exists and overwrite
    /// policy is Skip).
    #[error("Conflict: {0}")]
    Conflict(String),

    /// The command supplied by the user or external system is malformed.
    #[error("Invalid command: {0}")]
    InvalidCommand(String),

    /// The requested operation cannot be undone (e.g., already undone,
    /// not completed).
    #[error("Operation not undoable: {0}")]
    UndoNotPossible(String),

    /// An invalid state transition was attempted (e.g., completing a
    /// non-executing operation).
    #[error("Invalid state transition: {0}")]
    InvalidState(String),

    /// The operation with the given ID was not found.
    #[error("Operation not found: {0}")]
    OperationNotFound(String),

    /// A domain-level invariant was violated.
    /// The original domain error message is preserved for diagnostics.
    #[error("Domain error: {0}")]
    Domain(String),

    // ---- Infrastructure-level errors ----
    // These errors originate from external systems (disk, database, etc.).
    /// The repository failed to load or save data.
    #[error("Repository error: {0}")]
    RepositoryError(String),

    /// The file system returned an error (e.g., disk full, file locked).
    #[error("File system error: {0}")]
    FileSystemError(String),

    /// The destination volume does not have enough space for the operation.
    /// `need` is the total size of the sources that must be written,
    /// `available` is the free space on the target volume at check time.
    #[error("Not enough disk space on target: {need} bytes needed, {available} bytes available")]
    InsufficientDiskSpace { need: u64, available: u64 },

    // ---- Internal errors ----
    // These errors indicate unexpected states that should never occur
    // under normal operation.
    /// An unexpected internal error occurred. Use sparingly, only for
    /// truly unrecoverable situations (e.g., poisoned mutex, broken
    /// invariant).
    // English-only comment as per project standards
    #[error("Internal Use Case error: {0}")]
    Internal(String),

    /// An undo/repeat operation failed after the pre-flight checks
    /// passed. Carries a pre-classified [`OperationErrorDto`] so the
    /// adapter (Tauri command) can route the failure to the UI without
    /// re-mapping the underlying cause.
    #[error("undo failed: {0:?}")]
    UndoFailed(OperationErrorDto),
}

/// Classification of an undo/repeat failure for UI routing.
///
/// The kind is the authoritative discriminator: the frontend decides how
/// to treat the failure based on this value and never parses the
/// human-readable `message` text. Serialized lowercase to mirror the
/// TypeScript union `'transient' | 'permanent' | 'unknown'`.
///
/// - `Transient` — retrying may succeed (e.g. a file was momentarily
///   locked by another process). The action button stays enabled.
/// - `Permanent` — retrying cannot succeed until the user changes the
///   external state (operation gone, source file missing). The action
///   button is disabled in the UI.
/// - `Unknown` — the cause could not be determined reliably; fall back to
///   a generic toast and keep the action enabled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UndoErrorKind {
    Transient,
    Permanent,
    Unknown,
}

/// Wire-format error returned by undo/repeat commands to the frontend.
///
/// `kind` drives UI routing (button state, toast type); `message` is for
/// display only and must never be re-parsed by the UI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperationErrorDto {
    pub kind: UndoErrorKind,
    pub message: String,
}

impl UseCaseError {
    /// Maps this error to the wire-format DTO consumed by undo/repeat
    /// commands.
    ///
    /// Classification follows the P1-5 table:
    /// - `UndoFailed` passes its pre-classified DTO through unchanged.
    /// - `UndoNotPossible` is `Permanent` (retrying cannot succeed).
    /// - Everything else is `Unknown` — without `io::ErrorKind` the cause
    ///   cannot be determined reliably (e.g. an `exists()` probe failure
    ///   may be transient or permanent depending on the reason).
    pub fn to_operation_error(&self) -> OperationErrorDto {
        match self {
            UseCaseError::UndoFailed(dto) => dto.clone(),
            UseCaseError::UndoNotPossible(_) => OperationErrorDto {
                kind: UndoErrorKind::Permanent,
                message: self.to_string(),
            },
            _ => OperationErrorDto {
                kind: UndoErrorKind::Unknown,
                message: self.to_string(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repository_error_maps_to_unknown() {
        let err = UseCaseError::RepositoryError("repo down".to_string());
        let dto = err.to_operation_error();
        assert_eq!(dto.kind, UndoErrorKind::Unknown);
        assert_eq!(dto.message, "Repository error: repo down");
    }

    #[test]
    fn file_system_error_maps_to_unknown() {
        // A bare FileSystemError (e.g. an exists() probe failure) cannot be
        // classified without io::ErrorKind, so the UI must fall back to a
        // generic toast (P1-5).
        let err = UseCaseError::FileSystemError("probe failed".to_string());
        let dto = err.to_operation_error();
        assert_eq!(dto.kind, UndoErrorKind::Unknown);
    }

    #[test]
    fn undo_not_possible_maps_to_permanent() {
        let err = UseCaseError::UndoNotPossible("already undone".to_string());
        let dto = err.to_operation_error();
        assert_eq!(dto.kind, UndoErrorKind::Permanent);
        assert_eq!(dto.message, "Operation not undoable: already undone");
    }

    #[test]
    fn undo_failed_passes_dto_through_unchanged() {
        let dto = OperationErrorDto {
            kind: UndoErrorKind::Transient,
            message: "File system error: access denied".to_string(),
        };
        let mapped = UseCaseError::UndoFailed(dto.clone()).to_operation_error();
        assert_eq!(mapped, dto);
    }

    #[test]
    fn dto_serializes_kind_as_lowercase() {
        let dto = OperationErrorDto {
            kind: UndoErrorKind::Transient,
            message: "x".to_string(),
        };
        let json = serde_json::to_string(&dto).unwrap();
        assert!(json.contains("\"kind\":\"transient\""));
    }
}
