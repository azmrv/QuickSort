//! Domain errors – business rule violations.
//!
//! These errors are raised by domain entities and value objects when an
//! invariant is violated.  They are **not** meant to be presented directly
//! to end users; the Application layer maps them into `UseCaseError`.

use thiserror::Error;

/// Domain-level error type.
///
/// Every variant represents a specific business rule violation.
/// The `Display` implementation (derived via `thiserror`) provides a
/// human-readable description suitable for logging and debugging.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DomainError {
    // The provided path is an empty string.
    #[error("Path is empty")]
    EmptyPath,

    // The provided path contains invalid characters or structure.
    #[error("Invalid path: {0}")]
    InvalidPath(String),

    // The folder name was empty or consisted only of whitespace.
    #[error("Invalid folder name")]
    InvalidFolderName,

    // The operation attempted to use a root directory (e.g., `C:\`) as a
    /// target, which is not allowed for safety reasons.
    #[error("Illegal target directory (root)")]
    IllegalDirectoryTarget,

    // An invalid state transition was attempted on an `Operation` aggregate.
    #[error("Invalid operation state transition")]
    InvalidStateTransition,

    // The requested operation does not exist.
    #[error("Operation not found")]
    OperationNotFound,

    // The requested folder does not exist.
    #[error("Folder not found")]
    FolderNotFound,

    //  A business-level conflict occurred (e.g., duplicate folder name).
    #[error("Conflict: {0}")]
    Conflict(String),

    //  The operation was denied due to insufficient permissions.
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    // An unexpected internal error occurred within the domain.
    /// This should be used sparingly and only for truly exceptional situations.
    #[error("Internal domain error: {0}")]
    Internal(String),
}