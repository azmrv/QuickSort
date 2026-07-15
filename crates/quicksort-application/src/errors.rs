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

use thiserror::Error;

/// Unified error type for the Application layer.
///
/// All public methods in the Use Cases return `Result<T, UseCaseError>`.
/// Adapters (Tauri commands, IPC handlers) map these to appropriate
/// responses (e.g., HTTP status codes, user notifications, log entries).
///
/// # Example
/// ```rust
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

    // ---- Internal errors ----
    // These errors indicate unexpected states that should never occur
    // under normal operation.

    /// An unexpected internal error occurred. Use sparingly, only for
    /// truly unrecoverable situations (e.g., poisoned mutex, broken
    /// invariant).
    // OLD: комментарий на русском языке
    // NEW: English-only comment as per project standards
    #[error("Internal Use Case error: {0}")]
    Internal(String),
}