//! Inbound ports define the public API of the Application Layer.
//!
//! Adapters (GUI, CLI, Shell) depend on these interfaces, never on
//! concrete implementations. This follows the Dependency Inversion Principle:
//! high-level modules (adapters) depend on abstractions, not on details.
//!
//! # Ports Overview
//! | Port | Purpose | Implemented by |
//! |------|---------|----------------|
//! | `ExecuteOperation` | Move, Copy, Delete, Rename | `ExecuteOperationUseCase` |
//! | `UndoOperation` | Revert a completed operation | `UndoOperationUseCase` |
//! | `GetFolders` | List all configured folders | `GetFoldersUseCase` |
//! | `ManageFolders` | CRUD operations on folders | `ManageFoldersUseCase` |
//! | `ApplicationFacade` | Single entry point combining all ports | `ApplicationFacadeImpl` |

mod execute_operation;   // ExecuteOperation – execute file operations
mod undo_operation;      // UndoOperation – revert completed operations
mod get_folders;         // GetFolders – retrieve all configured folders
mod manage_folders;      // ManageFolders – add, remove, rename, toggle favorite
mod facade;              // ApplicationFacade – combined inbound port interface
mod facade_impl;         // ApplicationFacadeImpl – concrete facade implementation

// Re-export all port traits so that adapters can depend on them directly.
pub use execute_operation::ExecuteOperation;
pub use undo_operation::UndoOperation;
pub use get_folders::GetFolders;
pub use manage_folders::ManageFolders;
pub use facade::ApplicationFacade;
// OLD: pub use facade_impl::ApplicationFacadeImpl;
// NEW: ApplicationFacadeImpl is now re-exported for adapters that need
// to construct the facade (e.g., during dependency injection in main.rs).
pub use facade_impl::ApplicationFacadeImpl;

// ---------------------------------------------------------------------------
// Notes for future maintainers:
// - Adapters should depend only on these re-exported traits, never on
//   concrete use case types. Use `Arc<dyn ExecuteOperation>` instead of
//   `Arc<ExecuteOperationUseCase>` to allow easy mocking in tests.
// - When adding a new inbound port, create a file in this directory, define
//   the trait, implement it in a use case, add the use case to
//   ApplicationFacadeImpl, and add the re-export here.
// ---------------------------------------------------------------------------