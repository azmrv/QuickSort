# Inbound Ports Architecture

## What Are Inbound Ports?

Inbound ports are the **interface through which external actors** (GUI, CLI, Windows Shell Extension, future REST API) interact with the Application Layer. They define the **public API** of the application core.

In Hexagonal Architecture terms:

- **Inbound ports** = interfaces that **drive** the system (called by adapters).
- **Outbound ports** = interfaces that the system **drives** (implemented by infrastructure).

Inbound ports are defined in the **Application Layer** and implemented by **Use Cases** (which are part of the Application Layer as well). They are **not** implementations — they are contracts that adapters depend on.

---

## Design Principles

### 1. Single Responsibility

Each inbound port corresponds to one **cohesive set of operations**. For example, `ExecuteOperation` handles all file operations, while `GetFolders` handles read‑only queries.

### 2. Technology Agnostic

Inbound ports use only **domain and application types** (e.g., `OperationCommand`, `OperationResult`, `Folder`, `OperationId`). They never expose infrastructure types like `PathBuf`, `HashMap`, or `serde_json::Value`.

### 3. Asynchronous by Default

All inbound port methods are `async`. This allows adapters to call them without blocking the UI or the shell process. The implementation uses `tokio` or `async-std` under the hood.

### 4. Clear Error Boundaries

All errors returned from inbound ports are **application‑level errors** (`UseCaseError`). Adapters receive a structured error that can be mapped to user‑friendly messages or HTTP status codes.

### 5. Immutable Commands

Commands (input DTOs) are immutable. They are created by the adapter and passed to the port. The port does not modify them.

---

## Inbound Ports Defined

We define the following inbound ports in `crates/quicksort-application/src/ports/inbound/`.

### 1. `ExecuteOperation`

Handles all file operations: move, copy, delete, rename, and future extensions.

```rust
// File: crates/quicksort-application/src/ports/inbound/execute_operation.rs

use async_trait::async_trait;
use crate::dtos::{OperationCommand, OperationResult};
use crate::errors::UseCaseError;

/// Port for executing file operations.
/// Implemented by ExecuteOperationUseCase.
#[async_trait]
pub trait ExecuteOperation: Send + Sync {
    /// Execute the given operation command.
    /// Returns the result and a collection of domain events.
    async fn execute(&self, command: OperationCommand) -> Result<OperationResult, UseCaseError>;
}
```

### 2. `UndoOperation`

Allows rolling back a previously completed operation.

```rust
// File: crates/quicksort-application/src/ports/inbound/undo_operation.rs

use async_trait::async_trait;
use quicksort_domain::value_objects::OperationId;
use crate::dtos::OperationResult;
use crate::errors::UseCaseError;

/// Port for undoing a completed operation.
#[async_trait]
pub trait UndoOperation: Send + Sync {
    /// Reverse the effects of the operation identified by `operation_id`.
    async fn undo(&self, operation_id: OperationId) -> Result<OperationResult, UseCaseError>;
}
```

### 3. `GetFolders`

Reads the current folder configuration.

```rust
// File: crates/quicksort-application/src/ports/inbound/get_folders.rs

use async_trait::async_trait;
use quicksort_domain::entities::Folder;
use crate::errors::UseCaseError;

/// Port for retrieving folders from the configuration.
#[async_trait]
pub trait GetFolders: Send + Sync {
    /// Returns all configured folders.
    async fn get_all(&self) -> Result<Vec<Folder>, UseCaseError>;
}
```

### 4. `ManageFolders` (optional, for future)

Add, remove, rename, or reorder folders.

```rust
// File: crates/quicksort-application/src/ports/inbound/manage_folders.rs

use async_trait::async_trait;
use quicksort_domain::entities::Folder;
use quicksort_domain::value_objects::FolderId;
use crate::errors::UseCaseError;

#[async_trait]
pub trait ManageFolders: Send + Sync {
    async fn add_folder(&self, folder: Folder) -> Result<(), UseCaseError>;
    async fn remove_folder(&self, id: FolderId) -> Result<(), UseCaseError>;
    async fn rename_folder(&self, id: FolderId, new_name: String) -> Result<(), UseCaseError>;
    async fn toggle_favorite(&self, id: FolderId, order: i32) -> Result<(), UseCaseError>;
}
```

---

## Unified Application Facade

For convenience, we provide a **facade** that combines all inbound ports into a single trait. This simplifies dependency injection for adapters that need access to multiple ports.

```rust
// File: crates/quicksort-application/src/ports/inbound/facade.rs

use super::*;

/// Combined interface for all inbound operations.
pub trait ApplicationFacade:
    ExecuteOperation
    + UndoOperation
    + GetFolders
    + ManageFolders
    + Send
    + Sync
{
}

// Blanket implementation for any type that implements all required traits.
impl<T> ApplicationFacade for T where
    T: ExecuteOperation
        + UndoOperation
        + GetFolders
        + ManageFolders
        + Send
        + Sync
{
}
```

Adapters can then depend on `ApplicationFacade` instead of individual traits. This reduces the number of dependencies injected.

---

## DTOs (Data Transfer Objects)

Inbound ports use DTOs that are defined in the Application Layer. These DTOs are independent of domain entities and infrastructure.

### `OperationCommand`

```rust
// File: crates/quicksort-application/src/dtos/operation_command.rs

use quicksort_domain::value_objects::{FolderId, WindowsPath};
use quicksort_domain::OperationType;

/// Command to execute a file operation.
#[derive(Debug, Clone)]
pub struct OperationCommand {
    pub operation_type: OperationType,
    pub source_paths: Vec<WindowsPath>,
    pub target_folder_id: Option<FolderId>, // None for Delete or Rename
    pub overwrite_policy: OverwritePolicy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverwritePolicy {
    Skip,      // Do nothing if target exists
    Overwrite, // Replace existing file
    AutoRename,// Append suffix (e.g., "file (1).txt")
    Ask,       // Defer to user (handled by ConflictResolver)
}
```

### `OperationResult`

```rust
// File: crates/quicksort-application/src/dtos/operation_result.rs

use quicksort_domain::value_objects::OperationId;
use quicksort_domain::OperationState;

/// Result of a completed or failed operation.
#[derive(Debug, Clone)]
pub struct OperationResult {
    pub operation_id: OperationId,
    pub state: OperationState,
    pub processed_files: u32,
    pub bytes_moved: u64,
    // Additional details can be added (e.g., list of successful/failed paths)
}
```

---

## Error Handling for Inbound Ports

All inbound port methods return `Result<T, UseCaseError>`. The `UseCaseError` enum is defined in the Application Layer and covers all possible failures that can occur during Use Case execution.

```rust
// File: crates/quicksort-application/src/errors/use_case_error.rs

#[derive(Debug, thiserror::Error)]
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

    #[error("Internal error: {0}")]
    Internal(String),
}
```

Adapters are responsible for mapping these errors to user‑friendly messages (e.g., in the GUI) or appropriate HTTP status codes (if a REST API is added later).

---

## Dependency Injection

Inbound ports are **implemented by Use Cases**. The **composition root** (e.g., Tauri’s `setup()` hook or the main function) creates the Use Case instances, injects their outbound port dependencies, and registers them in the application state.

### Example: Tauri Adapter

```rust
// In src-tauri/src/main.rs (the composition root)

let file_system = Arc::new(Win32FileSystem::new());
let config_repo = Arc::new(JsonConfigurationRepository::new(config_path));
let op_repo = Arc::new(JsonOperationRepository::new(history_path));
let id_gen = Arc::new(UlidGenerator::new());
let clock = Arc::new(SystemClock);
let conflict_resolver = Arc::new(DefaultConflictResolver);

let execute_use_case = Arc::new(ExecuteOperationUseCase::new(
    config_repo.clone(),
    op_repo.clone(),
    file_system.clone(),
    id_gen.clone(),
    clock.clone(),
    conflict_resolver.clone(),
));

let undo_use_case = Arc::new(UndoOperationUseCase::new(
    op_repo.clone(),
    file_system.clone(),
    clock.clone(),
));

let get_folders_use_case = Arc::new(GetFoldersUseCase::new(config_repo.clone()));

let facade: Arc<dyn ApplicationFacade> = Arc::new(ApplicationFacadeImpl {
    execute: execute_use_case,
    undo: undo_use_case,
    get_folders: get_folders_use_case,
});

// Store the facade in Tauri state
tauri::Builder::default()
    .manage(facade)
    .invoke_handler(tauri::generate_handler![
        execute_command_handler,
        undo_operation_handler,
        get_folders_handler
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
```

### Tauri Command Handlers (Adapters)

```rust
// In src-tauri/src/adapters/tauri_cmd/mod.rs

#[tauri::command]
async fn execute_command_handler(
    state: tauri::State<'_, Arc<dyn ApplicationFacade>>,
    command: OperationCommand,
) -> Result<OperationResult, String> {
    state
        .execute(command)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn undo_operation_handler(
    state: tauri::State<'_, Arc<dyn ApplicationFacade>>,
    operation_id: String,
) -> Result<OperationResult, String> {
    let id = OperationId::from_string(operation_id);
    state
        .undo(id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_folders_handler(
    state: tauri::State<'_, Arc<dyn ApplicationFacade>>,
) -> Result<Vec<Folder>, String> {
    state
        .get_all()
        .await
        .map_err(|e| e.to_string())
}
```

---

## Security Considerations

1. **Input Validation** – The Application Layer performs validation on all incoming commands (e.g., checking that paths are absolute, that the target folder exists). However, adapters should also perform basic validation to avoid sending obviously invalid data.

2. **Authorization** – In the current version, QuickSort runs with the user’s permissions. No additional authorization checks are performed. In a future enterprise version, a security port could be added to verify that the user has permission to access certain paths.

3. **Path Traversal** – The `WindowsPath` validation (in the Domain Layer) prevents relative paths and ensures that only absolute paths are processed. This is a domain invariant.

---

## Testing Inbound Ports

Inbound ports are tested indirectly via **executable specifications** (see `executable-specification.md`). The specifications verify that the Use Case implementations behave correctly. The port itself is just a trait — it does not need its own tests.

However, adapters should have **integration tests** to verify that they correctly call the inbound ports and handle errors.

---

## Future Extensions

As the platform grows, new inbound ports may be added, such as:

- `ScheduleOperation` – for cron‑like scheduled tasks.
- `BatchOperation` – for processing multiple commands in one call.
- `QueryHistory` – for retrieving operation history.
- `ExportConfiguration` / `ImportConfiguration` – for backup and restore.

These ports will follow the same principles defined here.

---

## Summary

| Component | Responsibility | Location |
|-----------|----------------|----------|
| **Inbound Ports** | Define the public API of the Application Layer | `crates/quicksort-application/src/ports/inbound/` |
| **Use Cases** | Implement inbound ports | `crates/quicksort-application/src/use_cases/` |
| **DTOs** | Data structures for commands and results | `crates/quicksort-application/src/dtos/` |
| **Facade** | Combined interface for all ports (optional) | `crates/quicksort-application/src/ports/inbound/facade.rs` |
| **Adapters** | Call inbound ports (GUI, CLI, Shell) | `src-tauri/`, `context-menu-dll/`, `quicksort-cli/` |

---

## References

- [Hexagonal Architecture (Alistair Cockburn)](https://alistair.cockburn.us/hexagonal-architecture/)
- [Clean Architecture (Robert C. Martin)](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html)
- [Ports & Adapters Pattern](https://en.wikipedia.org/wiki/Hexagonal_architecture_(software))
