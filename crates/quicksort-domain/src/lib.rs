//! Domain layer – pure business logic, no external dependencies.

// Re-export all domain types directly from the root.
mod value_objects;
mod entities;
mod events;
mod errors;
mod operation;

pub use value_objects::*;
pub use entities::*;
pub use events::*;
pub use errors::*;
