# Domain Models & Invariants Specification

This document defines the core aggregates, entities, and value objects within the pure Domain layer. These structures are business‑rule driven and have zero technical dependencies (no `uuid`, `chrono`, `serde`, etc.). All identifiers are plain strings; generation is delegated to the `IdGenerator` port (infrastructure).

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
```

---

## 2. Entities

### `Folder`

A folder is a configuration asset. It has a name, a path, and a favourite flag. It is not an aggregate root; it is managed by the configuration repository.

```rust
#[derive(Debug, Clone)]
pub struct Folder {
    pub id: FolderId,
    pub name: String,
    pub path: WindowsPath,
    pub is_favorite: bool,
    pub sort_order: i32,
}

impl Folder {
    pub fn new(id: FolderId, name: String, path: WindowsPath) -> Self {
        Self {
            id,
            name,
            path,
            is_favorite: false,
            sort_order: 0,
        }
    }

    /// Business invariant: a folder cannot be a root drive (e.g., "C:\").
    /// This prevents the user from accidentally selecting the entire drive.
    pub fn validate(&self) -> Result<(), DomainError> {
        if self.path.is_root() {
            return Err(DomainError::IllegalDirectoryTarget);
        }
        Ok(())
    }

    /// Toggles the favourite status and updates the sort order.
    pub fn toggle_favorite(&mut self, order: i32) {
        self.is_favorite = !self.is_favorite;
        self.sort_order = if self.is_favorite { order } else { 0 };
    }

    /// Renames the folder.
    pub fn rename(&mut self, new_name: String) {
        self.name = new_name;
    }
}
```

### `Operation` (Aggregate Root)

The operation is the central entity of the system. It tracks its own state machine, lifecycle, and collects domain events for later dispatch.

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperationType {
    Move,
    Copy,
    Delete,
    Rename,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperationState {
    Pending,
    Executing,
    Completed {
        processed_files: u32,
        bytes_moved: u64,
    },
    Failed {
        reason: String,
    },
    Undone,
}

#[derive(Debug, Clone)]
pub struct Operation {
    pub id: OperationId,
    pub operation_type: OperationType,
    pub state: OperationState,
    pub source_paths: Vec<WindowsPath>,
    pub target_folder_path: Option<WindowsPath>, // None for Delete operations
    pub created_at: std::time::SystemTime,
    pub updated_at: std::time::SystemTime,
    events: Vec<DomainEvent>,
}

impl Operation {
    /// Creates a new operation in the Pending state.
    pub fn new(
        id: OperationId,
        op_type: OperationType,
        source_paths: Vec<WindowsPath>,
        target: Option<WindowsPath>,
        now: std::time::SystemTime,
    ) -> Self {
        Self {
            id,
            operation_type: op_type,
            state: OperationState::Pending,
            source_paths,
            target_folder_path: target,
            created_at: now,
            updated_at: now,
            events: Vec::new(),
        }
    }

    /// Clears and returns all collected domain events (Clear-on-Read Pattern).
    pub fn pull_events(&mut self) -> Vec<DomainEvent> {
        std::mem::take(&mut self.events)
    }

    /// Returns true if the operation can be undone (only when Completed).
    pub fn can_undo(&self) -> bool {
        matches!(self.state, OperationState::Completed { .. })
    }

    /// Transitions the operation to Executing state.
    pub fn start(&mut self, now: std::time::SystemTime) -> Result<(), DomainError> {
        if !matches!(self.state, OperationState::Pending) {
            return Err(DomainError::InvalidStateTransition);
        }
        self.state = OperationState::Executing;
        self.updated_at = now;

        self.events.push(DomainEvent::OperationStarted {
            operation_id: self.id.clone(),
            op_type: self.operation_type.clone(),
        });
        Ok(())
    }

    /// Transitions the operation to Completed state.
    pub fn complete(
        &mut self,
        files: u32,
        bytes: u64,
        now: std::time::SystemTime,
    ) -> Result<(), DomainError> {
        if !matches!(self.state, OperationState::Executing) {
            return Err(DomainError::InvalidStateTransition);
        }
        self.state = OperationState::Completed {
            processed_files: files,
            bytes_moved: bytes,
        };
        self.updated_at = now;

        self.events.push(DomainEvent::OperationCompleted {
            operation_id: self.id.clone(),
            files,
            bytes,
        });
        Ok(())
    }

    /// Transitions the operation to Failed state.
    pub fn fail(&mut self, reason: String, now: std::time::SystemTime) -> Result<(), DomainError> {
        if !matches!(self.state, OperationState::Pending | OperationState::Executing) {
            return Err(DomainError::InvalidStateTransition);
        }
        self.state = OperationState::Failed { reason: reason.clone() };
        self.updated_at = now;

        self.events.push(DomainEvent::OperationFailed {
            operation_id: self.id.clone(),
            reason,
        });
        Ok(())
    }

    /// Transitions the operation to Undone state.
    pub fn mark_undone(&mut self, now: std::time::SystemTime) -> Result<(), DomainError> {
        if !matches!(self.state, OperationState::Completed { .. }) {
            return Err(DomainError::InvalidStateTransition);
        }
        self.state = OperationState::Undone;
        self.updated_at = now;

        self.events.push(DomainEvent::OperationUndone {
            operation_id: self.id.clone(),
        });
        Ok(())
    }
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

Core domain errors representing business rule violations.

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainError {
    EmptyPath,
    InvalidPath(String),
    IllegalDirectoryTarget,
    InvalidStateTransition,
}
```

---

## 5. Why This Design

- **Zero external dependencies** – the domain crate contains only `std` and its own types. No `uuid`, `chrono`, `serde`, or any framework. This makes it trivial to test and reason about.
- **Plain identifiers** – `FolderId` and `OperationId` are strings. Generation is injected via a port, which allows swapping between UUID, ULID, or sequential IDs without touching domain code.
- **State machine** – `Operation` has explicit state transitions (`start` → `complete` / `fail` → `undone`). This makes the lifecycle explicit and prevents invalid states.
- **Domain events** – all events are in one enum. The Application layer produces these events based on state changes and returns them to the infrastructure.
- **Event collection inside aggregate** – the operation collects events during state changes. The `pull_events()` method clears them, following the clear-on-read pattern.

---

## 6. Next Steps

With the domain models defined, we proceed to:

1. Implement the domain crate with these structures.
2. Define Application layer **ports** (interfaces) that use these models.
3. Write **Executable Specifications** (scenario tests) for all Use Cases.
4. Define **Inbound Ports** (API for adapters).
5. Implement Infrastructure adapters (JSON repository, file system, etc.).

---
