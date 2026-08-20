// quicksort-application/src/lib.rs

//! Application Layer of QuickSort.
//! This module exposes the Application Facade, which orchestrates all Use Cases.

pub mod use_cases;
pub mod ports;
pub mod dtos;
pub mod errors;

// Re-export core components for easy consumption by consumers (like Tauri or DLL adapters)
pub use use_cases::ExecuteOperationUseCase; // Example re-export

// Ensure all necessary public items from submodules are exposed correctly.
// If specific structs/traits need to be accessible globally, they must be explicitly re-exported here.