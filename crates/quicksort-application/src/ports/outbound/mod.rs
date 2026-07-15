//! Outbound ports are implemented by infrastructure.
//!
//! These ports define the interfaces that the Application Layer requires
//! from the outside world. They follow the Dependency Inversion Principle:
//! the Application Layer defines what it needs, and the Infrastructure Layer
//! provides the concrete implementations.
//!
//! # Ports Overview
//! | Port | Purpose | Used by |
//! |------|---------|---------|
//! | `ConfigurationRepository` | CRUD operations for folder config | `GetFoldersUseCase`, `ManageFoldersUseCase` |
//! | `OperationRepository` | Persist/retrieve operation history | `ExecuteOperationUseCase`, `UndoOperationUseCase` |
//! | `FileSystem` | File I/O (move, copy, delete, rename) | `ExecuteOperationUseCase`, `UndoOperationUseCase` |
//! | `IdGenerator` | Generate unique operation IDs | `ExecuteOperationUseCase` |
//! | `Clock` | Obtain current timestamp | `ExecuteOperationUseCase`, `UndoOperationUseCase` |
//! | `ConflictResolver` | Resolve file conflicts interactively | `ExecuteOperationUseCase` (future) |

mod configuration_repository;   // ConfigurationRepository – folder CRUD
mod operation_repository;      // OperationRepository – operation history persistence
mod file_system;               // FileSystem – file I/O operations
mod id_generator;              // IdGenerator – unique identifier generation
mod clock;                     // Clock – current time
mod conflict_resolver;         // ConflictResolver – interactive conflict resolution

// Re-export all port traits for use by other modules.
pub use configuration_repository::ConfigurationRepository;
pub use operation_repository::OperationRepository;
pub use file_system::FileSystem;
pub use id_generator::IdGenerator;
pub use clock::Clock;
pub use conflict_resolver::ConflictResolver;

// ---------------------------------------------------------------------------
// Notes for Infrastructure implementors:
// - Each port is a trait with `Send + Sync` bounds, suitable for use with `Arc`.
// - Implementations should live in `quicksort-infrastructure` and be injected
//   into the application via dependency injection (see `main.rs` or `setup`).
// - All error types returned by ports must be mapped to `UseCaseError` to keep
//   the Application layer independent of infrastructure-specific error types
//   (e.g., `std::io::Error`, `serde_json::Error`).
// ---------------------------------------------------------------------------