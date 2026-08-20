// Synthesized content for quicksort-domain/src/value_objects/mod.rs
use std::fmt;

// --- Domain Errors (Copied for completeness, assuming it exists) ---
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainError {
    EmptyPath,
    InvalidPath(String),
    IllegalDirectoryTarget,
    InvalidStateTransition,
}

// --- Value Objects ---

/// A sanitised absolute Windows path. Validation happens at construction to prevent invalid states from entering the domain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowsPath(String);

impl WindowsPath {
    /// Creates a new WindowsPath from a string.
    /// Replaces forward slashes with backslashes and validates that the path is absolute.
    pub fn new(path: &str) -> Result<Self, DomainError> {
        let mut sanitized = path.replace('/', "\\\\");
        if sanitized.is_empty() {
            return Err(DomainError::EmptyPath);
        }

        // Validate UNC paths (e.g., "\\\\server\\share")
        if sanitized.starts_with("\\\\\\\\") {
            if sanitized.len() == 2 {
                return Err(DomainError::InvalidPath("Invalid UNC path".to_string()));
            }
            return Ok(Self(sanitized));
        }

        // Validate classic Windows drive paths (e.g., "C:\\")
        let chars: Vec<char> = sanitized.chars().collect();
        if chars.len() >= 2 && chars[0].is_ascii_alphabetic() && chars[1] == ':' {
            // Enforce trailing backslash for root drives during sanitisation
            if chars.len() == 2 {
                sanitized.push('\\\\');
            } else if chars[2] != '\\\\' {
                return Err(DomainError::InvalidPath("Drive letter must be followed by a backslash".to_string()));
            }
            return Ok(Self(sanitized));
        }

        Err(DomainError::InvalidPath("Path must be absolute (UNC or drive letter)".to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns true if the path is a root drive (e.g., "C:\\").
    pub fn is_root(&self) -> bool {
        // A valid root drive always maps to exactly 3 characters (e.g., "C:\\\")
        self.0.len() == 3 && self.0.ends_with(":\\\\")
    }
}

/// Identifiers are plain strings. The domain does not generate them; generation is injected via the `IdGenerator` port.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FolderId(String);

impl FolderId {
    pub fn from_string(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OperationId(String);

impl OperationId {
    pub fn from_string(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}


// --- Entities ---

/// A folder is a configuration asset. It has a name, a path, and a favourite flag. It is not an aggregate root; it is managed by the configuration repository.
#[derive(Debug, Clone)]
pub struct Folder {
    pub id: FolderId,
    pub name: String,
    pub path: WindowsPath,
    pub is_favorite: bool,
    pub operation_id: OperationId, // Link to the current operation context
    pub updated_at: std::time::SystemTime,
    pub state: OperationState,
    pub events: Vec<DomainEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperationState {
    Pending,
    Completed { files: u32, bytes: u64 },
    Failed { reason: String },
    Undone,
}


// --- Domain Events ---

/// All domain events are collected in a single enum. This makes handling straightforward and keeps the domain clean. Events are immutable and carry all necessary context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainEvent {
    /// Emitted when an operation starts execution.
    OperationStarted {
        operation_id: OperationId,
        op_type: OperationType, // Assuming OperationType is defined elsewhere or injected here
    },
    /// Emitted when an operation completes successfully.
    OperationCompleted {
        operation_id: OperationId,
        files: u32,
        bytes: u64,
    },
    /// Emitted when an operation fails.
    OperationFailed {
        operation_id: OperationId,
        reason: String,
    },
    /// Emitted when an operation is successfully rolled back.
    OperationUndone {
        operation_id: OperationId,
    },
    // ... (other events omitted for brevity)
}

// --- Domain Errors ---

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainError {
    EmptyPath,
    InvalidPath(String),
    IllegalDirectoryTarget,
    InvalidStateTransition,
}