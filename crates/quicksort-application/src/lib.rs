//! Application Layer of QuickSort.
//!
//! This crate contains:
//! - Use Cases (business process orchestration)
//! - Ports (interfaces for adapters and infrastructure)
//! - DTOs (data transfer objects for commands and results)
//! - Errors (application-level error types)
//!
//! It depends only on the Domain crate and external traits (async-trait, thiserror).
//! It does NOT depend on any infrastructure or UI frameworks.

pub mod errors;
pub mod dtos;
pub mod ports;
pub mod use_cases;

// Re-export commonly used items for convenience.
pub use errors::UseCaseError;
pub use dtos::{OperationCommand, OperationResult, OverwritePolicy};
pub use ports::inbound::{
    ExecuteOperation,
    // UndoOperation,
    GetFolders,
    ManageFolders,
    ApplicationFacade,
};