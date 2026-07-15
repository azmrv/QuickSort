//! QuickSort Domain Layer
//!
//! This crate defines the core business entities, value objects, and events
//! for the QuickSort file management application.

pub mod dtos; // Data Transfer Objects - перемещены сюда из application layer
pub mod entities;
pub mod errors;
pub mod events;
pub mod value_objects;

pub use dtos::{OperationCommand, OperationResult, OverwritePolicy, create_operation_failure};
pub use entities::{Folder, Operation, OperationState, OperationType};
pub use errors::DomainError;
pub use events::DomainEvent;
pub use value_objects::{WindowsPath, FolderId, OperationId};