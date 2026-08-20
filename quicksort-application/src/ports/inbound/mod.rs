// quicksort-application/src/ports/inbound/mod.rs

pub mod execute_operation;      // ExecuteOperation – execute file operations
pub mod operation_repository;      // OperationRepository – operation history persistence
pub mod conflict_resolver;         // ConflictResolver – interactive conflict resolution

// Re-export the main interface for external consumers (Facade)
pub use execute_operation::ExecuteOperation;
pub use operation_repository::OperationRepository;
pub use conflict_resolver::ConflictResolver;