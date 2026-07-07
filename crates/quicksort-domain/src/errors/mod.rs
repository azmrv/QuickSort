//! Domain errors – business rule violations.
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainError {
    EmptyPath,
    InvalidPath(String),
    IllegalDirectoryTarget,
    InvalidStateTransition,
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DomainError::EmptyPath => write!(f, "Path is empty"),
            DomainError::InvalidPath(s) => write!(f, "Invalid path: {}", s),
            DomainError::IllegalDirectoryTarget => write!(f, "Illegal directory target"),
            DomainError::InvalidStateTransition => write!(f, "Invalid state transition"),
        }
    }
}

impl std::error::Error for DomainError {}