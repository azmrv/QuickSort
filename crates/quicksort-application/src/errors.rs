//! Application-level error types.
//! All Use Cases return errors of this type.
//! Adapters map these to user-friendly messages.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum UseCaseError {
    #[error("Folder not found: {0}")]
    FolderNotFound(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Invalid command: {0}")]
    InvalidCommand(String),

    #[error("Repository error: {0}")]
    RepositoryError(String),

    #[error("File system error: {0}")]
    FileSystemError(String),

    #[error("Operation not undoable: {0}")]
    UndoNotPossible(String),

    #[error("Invalid state transition: {0}")]
    InvalidState(String),

    #[error("Operation not found: {0}")]
    OperationNotFound(String),

    #[error("Domain error: {0}")]
    Domain(String),

    /// Внутренняя ошибка использования случая (использовать с осторожностью)
    #[error("Internal Use Case error: {0}")]
    Internal(String),
}