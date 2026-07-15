//! Application Layer of QuickSort.
//!
//! This crate orchestrates business processes by coordinating domain entities
//! and infrastructure services. It defines:
//! - **Use Cases**: business logic workflows (e.g., `ExecuteOperation`, `UndoOperation`)
//! - **Ports**: interfaces for incoming (inbound) and outgoing (outbound) dependencies
//! - **DTOs**: plain data structures to transfer commands and results across boundaries
//! - **Errors**: application-level error types that map to user-friendly messages
//! - **Facade**: `ApplicationFacadeImpl` – the single entry point for all adapters
//!
//! # Architectural Guarantees
//! - Depends **only** on `quicksort-domain` and standard async traits (`async-trait`, `thiserror`).
//! - Does **not** import any infrastructure or UI frameworks (Tauri, Windows API, etc.).
//! - All external interactions go through the Port interfaces, following Hexagonal Architecture.
//!
//! # Re-export Policy
//! Only the types that are directly consumed by adapters (Tauri, CLI, IPC) are re-exported
//! at the crate root. Internal types (e.g., individual ports, use case implementations) should
//! be accessed through the full path to avoid polluting the namespace and to keep dependency
//! direction clear.
//!
//! # Module Organization
//! | Module | Purpose |
//! |--------|---------|
//! | `errors` | Unified error type (`UseCaseError`) |
//! | `dtos` | Data Transfer Objects (`OperationCommand`, `OperationResult`, etc.) |
//! | `ports` | Inbound and outbound interfaces |
//! | `use_cases` | Concrete implementations of inbound ports |
//! | `pipeline` | Optional command processing pipeline (validation, logging) |
//!
//! # Example – Creating the facade
//! ```rust,no_run
//! use quicksort_application::ApplicationFacadeImpl;
//! // ... create infrastructure services ...
//! let facade = ApplicationFacadeImpl::new(execute, undo, get_folders, manage);
//! ```

pub mod errors;
pub mod dtos;
pub mod ports;
pub mod use_cases;
pub mod pipeline;

// ---------------------------------------------------------------------------
// Re-export the public API – these are the only types adapters should depend on
// ---------------------------------------------------------------------------

// Error type – all Use Case operations return this error.
pub use errors::UseCaseError;

// DTOs – used by adapters to send commands and receive results.
pub use dtos::{OperationCommand, OperationResult, OverwritePolicy};

// Inbound ports – the contracts that adapters call.
pub use ports::inbound::{
    ExecuteOperation,
    UndoOperation,
    GetFolders,
    ManageFolders,
    ApplicationFacade,
    ApplicationFacadeImpl,
};

// Pipeline is intentionally NOT re-exported – it is an internal mechanism
// that adapters should not depend on directly.  They should call the facade,
// which may use the pipeline internally.

// ---------------------------------------------------------------------------
// Notes for future maintainers:
// - Adapters should depend only on the traits re-exported above, never on
//   concrete use case types.  Use `Arc<dyn ExecuteOperation>` for flexibility.
// - When adding a new inbound port, define the trait, implement it in a use
//   case, add the use case to `ApplicationFacadeImpl`, and then add the
//   re-export here.
// - The `pipeline` module is an internal implementation detail of the
//   Application Layer and should NOT be re-exported.
// ---------------------------------------------------------------------------

