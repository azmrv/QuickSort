# Domain Models & Invariants Specification

This document defines the core aggregates, entities, and value objects within the Domain layer. These structures are business‑rule driven and depend only on well-known crates (`chrono`, `serde`, `thiserror`) that do not impose architectural constraints. All identifiers are plain strings; generation is delegated to the `IdGenerator` port (infrastructure).

---

## 1. Value Objects

Value objects are immutable and defined solely by their attributes. They encapsulate validation logic.

### `WindowsPath`

A sanitised absolute Windows path. Validation happens at construction to prevent invalid states from entering the domain.

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowsPath(String);

impl WindowsPath {
    /// Creates a new WindowsPath from a string.
    /// Replaces forward slashes with backslashes and validates that the path is absolute.
    pub fn new(path: &str) -> Result<Self, DomainError> {
        let mut sanitized = path.replace('/', "\\");
        if sanitized.is_empty() {
            return Err(DomainError::EmptyPath);
        }

        // Validate UNC paths (e.g., "\\server\share")
        if sanitized.starts_with("\\\\") {
            if sanitized.len() == 2 {
                return Err(DomainError::InvalidPath("Invalid UNC path".to_string()));
            }
            return Ok(Self(sanitized));
        }

        // Validate classic Windows drive paths (e.g., "C:\")
        let chars: Vec<char> = sanitized.chars().collect();
        if chars.len() >= 2 && chars[0].is_ascii_alphabetic() && chars[1] == ':' {
            // Enforce trailing backslash for root drives during sanitisation
            if chars.len() == 2 {
                sanitized.push('\\');
            } else if chars[2] != '\\' {
                return Err(DomainError::InvalidPath("Drive letter must be followed by a backslash".to_string()));
            }
            return Ok(Self(sanitized));
        }

        Err(DomainError::InvalidPath("Path must be absolute (UNC or drive letter)".to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns true if the path is a root drive (e.g., "C:\").
    pub fn is_root(&self) -> bool {
        // A valid root drive always maps to exactly 3 characters (e.g., "C:\")
        self.0.len() == 3 && self.0.ends_with(":\\")
    }
}
```

### `FolderId` and `OperationId`

Identifiers are plain strings. The domain does not generate them; generation is injected via the `IdGenerator` port. This keeps the domain independent of any specific ID generation strategy (UUID, ULID, snowflake, etc.).

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FolderId(String);

impl FolderId {
    pub fn new() -> Self { /* generates UUID v7 */ }
    pub fn from_string(s: impl Into<String>) -> Self { Self(s.into()) }
    pub fn as_str(&self) -> &str { &self.0 }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OperationId(String);

impl OperationId {
    pub fn new() -> Self { /* generates UUID v7 */ }
    pub fn from_string(s: impl Into<String>) -> Self { Self(s.into()) }
    pub fn as_str(&self) -> &str { &self.0 }
}
```

---

## 2. Entities

### `Folder`

A folder is a configuration asset. It has a name, a path, usage statistics, and timestamps. It is not an aggregate root; it is managed by the configuration repository.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Folder {
    pub id: FolderId,
    pub name: String,
    pub path: WindowsPath,
    #[serde(default)]
    pub favorite: bool,
    #[serde(default)]
    pub order: u32,
    #[serde(default)]
    pub stats: FolderStats,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FolderStats {
    pub use_count: u64,
    pub last_used: Option<DateTime<Utc>>,
}

impl Folder {
    /// Creates a new folder with a generated ID and current timestamps.
    pub fn new(name: impl Into<String>, path: WindowsPath) -> Self { /* ... */ }

    /// Creates a new folder with an explicit ID (useful for testing).
    pub fn with_id(id: FolderId, name: impl Into<String>, path: WindowsPath) -> Self { /* ... */ }

    /// Updates the folder name. Returns DomainError::InvalidFolderName if empty.
    pub fn update_name(&mut self, name: impl Into<String>) -> Result<(), DomainError> { /* ... */ }

    /// Updates the folder path. Returns DomainError::IllegalDirectoryTarget if root.
    pub fn update_path(&mut self, new_path: WindowsPath) -> Result<(), DomainError> { /* ... */ }

    /// Toggles the favorite status.
    pub fn toggle_favorite(&mut self) { /* ... */ }

    /// Records that this folder was used for an operation.
    pub fn record_usage(&mut self) { /* ... */ }
}
```

### `Operation` (Aggregate Root)

The operation is the central entity of the system. It tracks its own state machine, lifecycle, and collects domain events for later dispatch.

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationType {
    Move,
    Copy,
    Delete,
    Rename,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationState {
    Pending,
    Executing,
    Completed {
        processed_files: u32,
        bytes_processed: u64,  // renamed from bytes_moved for Copy support
    },
    Failed {
        reason: String,
    },
    Undone,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    pub id: OperationId,
    pub operation_type: OperationType,
    pub state: OperationState,
    pub source_paths: Vec<WindowsPath>,
    pub target_folder_path: Option<WindowsPath>,    // for Move/Copy
    pub target_paths: Option<Vec<WindowsPath>>,      // for Rename
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip)]
    pub(crate) events: Vec<DomainEvent>,
}

impl Operation {
    pub fn new(id, op_type, source_paths, target, target_paths, now: DateTime<Utc>) -> Self { /* ... */ }

    /// Factory methods for each operation type:
    pub fn new_move(source: Vec<WindowsPath>, target: WindowsPath, now: DateTime<Utc>) -> Self { /* ... */ }
    pub fn new_copy(source: Vec<WindowsPath>, target: WindowsPath, now: DateTime<Utc>) -> Self { /* ... */ }
    pub fn new_delete(source: Vec<WindowsPath>, now: DateTime<Utc>) -> Self { /* ... */ }
    pub fn new_rename(source: Vec<WindowsPath>, target: WindowsPath, now: DateTime<Utc>) -> Self { /* ... */ }

    pub fn pull_events(&mut self) -> Vec<DomainEvent> { /* ... */ }

    // State transitions — no `now` parameter, uses Utc::now() internally:
    pub fn start(&mut self) -> Result<(), DomainError> { /* ... */ }
    pub fn complete(&mut self, files: u32, bytes: u64) -> Result<(), DomainError> { /* ... */ }
    pub fn fail(&mut self, reason: String) -> Result<(), DomainError> { /* ... */ }
    pub fn mark_undone(&mut self) -> Result<(), DomainError> { /* ... */ }
}
```

---

## 3. Domain Events

All domain events are collected in a single enum. This makes handling straightforward and keeps the domain clean. Events are immutable and carry all necessary context.

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainEvent {
    /// Emitted when an operation starts execution.
    OperationStarted {
        operation_id: OperationId,
        op_type: OperationType,
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
    /// Emitted when files are moved (future extension).
    // FilesMoved {
    //     source_paths: Vec<WindowsPath>,
    //     destination_path: WindowsPath,
    // },
    /// Emitted when a folder is added to configuration (future extension).
    // FolderAdded {
    //     folder_id: FolderId,
    //     name: String,
    //     path: WindowsPath,
    // },
}
```

---

## 4. Domain Errors

Core domain errors representing business rule violations. Uses `thiserror` for Display derivation.

```rust
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DomainError {
    #[error("Path is empty")]
    EmptyPath,

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("Invalid folder name")]
    InvalidFolderName,

    #[error("Illegal target directory (root)")]
    IllegalDirectoryTarget,

    #[error("Invalid operation state transition")]
    InvalidStateTransition,

    #[error("Operation not found")]
    OperationNotFound,

    #[error("Folder not found")]
    FolderNotFound,

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Internal domain error: {0}")]
    Internal(String),
}
```

---

## 5. Why This Design

- **Minimal external dependencies** – the domain crate depends only on well-known crates (`chrono`, `serde`, `thiserror`) that do not impose architectural constraints. No `uuid`, no framework dependencies.
- **Plain identifiers** – `FolderId` and `OperationId` are strings. Generation is injected via a port, which allows swapping between UUID, ULID, or sequential IDs without touching domain code.
- **State machine** – `Operation` has explicit state transitions (`start` → `complete` / `fail` → `undone`). This makes the lifecycle explicit and prevents invalid states.
- **Domain events** – all events are in one enum. The Application layer produces these events based on state changes and returns them to the infrastructure.
- **Event collection inside aggregate** – the operation collects events during state changes. The `pull_events()` method clears them, following the clear-on-read pattern.
- **Serde support** – entities derive `Serialize`/`Deserialize` for JSON persistence, while keeping domain logic pure.

---

## 6. Next Steps

With the domain models defined, we proceed to:

1. Implement the domain crate with these structures.
2. Define Application layer **ports** (interfaces) that use these models.
3. Write **Executable Specifications** (scenario tests) for all Use Cases.
4. Define **Inbound Ports** (API for adapters).
5. Implement Infrastructure adapters (JSON repository, file system, etc.).

---
