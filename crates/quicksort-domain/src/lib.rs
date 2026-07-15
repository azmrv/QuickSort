//! QuickSort Domain Layer
//!
//! This crate defines the core business entities, value objects, and events
//! for the QuickSort file management application. It follows Domain-Driven
//! Design principles and is the innermost circle in the Clean Architecture.
//!
//! # Invariants
//! - The Domain layer MUST NOT depend on any other project crate.
//! - It MAY depend on standard libraries and well-known crates
//!   (`serde`, `uuid`, `chrono`, `thiserror`) that do not impose
//!   architectural constraints.

// OLD: pub mod dtos;
// DTOs (OperationCommand, OperationResult, OverwritePolicy) are part of
// the Application layer's contract with the outside world.  They were
// temporarily placed here during an early refactoring phase but have
// been moved back to `quicksort-application`.  Removing them from the
// domain crate restores the Dependency Rule.
//
// If any downstream code still imports these types from `quicksort-domain`,
// update those imports to `quicksort-application`.

pub mod entities;
pub mod errors;
pub mod events;
pub mod value_objects;

pub use entities::{Folder, Operation, OperationState, OperationType};
pub use errors::DomainError;
pub use events::DomainEvent;
pub use value_objects::{WindowsPath, FolderId, OperationId};

// OLD: pub use dtos::{OperationCommand, OperationResult, OverwritePolicy, create_operation_failure};
// NEW: DTOs are no longer re-exported from the domain layer.
// Adapters and Application should obtain them from `quicksort-application`.У нас есть исправленный доменный слой, где DTO удалены. Теперь нужно убедиться, что Application слой правильно определяет и экспортирует эти DTO, и что все ссылки в адаптерах (Tauri команды, IPC) обновлены. Следующий шаг — проверить `quicksort-application` и обновить импорты во всех крейтах, которые использовали `quicksort-domain::OperationCommand`. Готовы продолжить?