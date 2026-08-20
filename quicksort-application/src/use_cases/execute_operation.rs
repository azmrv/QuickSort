// quicksort-application/src/use_cases/execute_operation.rs
use crate::ports::outbound::conflict_resolver::ConflictResolver;
use quicksort_domain::value_objects::windows_path::WindowsPath; // Assuming this is now public or correctly imported
use quicksort_domain::{Operation, OperationId};

// Make the internal module public if it needs to be used outside the module
pub mod conflict_resolver; 
pub use conflict_resolver::ConflictResolver;


// ... rest of the file ...