//! Outbound ports are implemented by infrastructure.
//! They represent dependencies that the Application Layer needs.

mod configuration_repository;
mod operation_repository;
mod file_system;
mod id_generator;
mod clock;
mod conflict_resolver;

pub use configuration_repository::*;
pub use operation_repository::*;
pub use file_system::*;
pub use id_generator::*;
pub use clock::*;
pub use conflict_resolver::*;