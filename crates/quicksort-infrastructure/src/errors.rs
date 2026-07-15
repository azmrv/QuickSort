//! Infrastructure-level error types.
//! Ошибки уровня инфраструктуры, возникающие во время операций файловой системы, хранения и т.д.
//! Эти ошибки преобразуются в UseCaseError адаптерами.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum InfrastructureError {
    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Repository error: {0}")]
    Repository(String),

    #[error("Path validation failed: {0}")]
    InvalidPath(String),

    #[error("UUID generation failed: {0}")]
    UuidGeneration(String),

    #[error("Clock operation failed: {0}")]
    Clock(String),

    #[error("File system error: {0}")]
    FileSystem(String),

    #[error("Internal infrastructure error: {0}")]
    Internal(String),
}

/// Конвертер инфраструктурных ошибок в UseCaseError.
/// Этот trait должен быть реализован для каждого адаптера.
pub trait ErrorConverter: Send + Sync {
    fn convert_error(&self, error: InfrastructureError) -> quicksort_application::errors::UseCaseError;
}

impl ErrorConverter for () {
    fn convert_error(&self, error: InfrastructureError) -> quicksort_application::errors::UseCaseError {
        use quicksort_application::errors::UseCaseError;

        match error {
            InfrastructureError::FileNotFound(path) => UseCaseError::FileNotFound(path),
            InfrastructureError::PermissionDenied(path) => UseCaseError::PermissionDenied(path),
            InfrastructureError::Io(io_error) => UseCaseError::FileSystemError(io_error.to_string()),
            InfrastructureError::Serialization(msg) => UseCaseError::RepositoryError(msg),
            InfrastructureError::Repository(msg) => UseCaseError::RepositoryError(msg),
            InfrastructureError::InvalidPath(path) => UseCaseError::FileNotFound(path),
            InfrastructureError::UuidGeneration(msg) => UseCaseError::Internal(msg),
            InfrastructureError::Clock(msg) => UseCaseError::Internal(msg),
            InfrastructureError::FileSystem(msg) => UseCaseError::FileSystemError(msg),
            InfrastructureError::Internal(msg) => UseCaseError::Internal(msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversion() {
        let converter = ();
        let result = converter.convert_error(InfrastructureError::FileNotFound("test".to_string()));
        match result {
            UseCaseError::FileNotFound(msg) => assert_eq!(msg, "test"),
            _ => panic!("Expected FileNotFound"),
        }
    }
}