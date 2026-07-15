//! Infrastructure implementations of outbound ports.

pub mod errors;
pub mod repository;
pub mod filesystem;
pub mod id_generator;
pub mod clock;
pub mod conflict_resolver;

// Re-export commonly used implementations.
pub use errors::{InfrastructureError, ErrorConverter};
pub use filesystem::StdFileSystem;
pub use id_generator::UuidGenerator;
pub use clock::SystemClock;
pub use conflict_resolver::DefaultConflictResolver;
pub use repository::{JsonConfigurationRepository, JsonOperationRepository};