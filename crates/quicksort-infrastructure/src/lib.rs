//! Infrastructure implementations of outbound ports.

pub mod clock;
pub mod conflict_resolver;
pub mod errors;
pub mod filesystem;
pub mod id_generator;
pub mod repository;

// Re-export commonly used implementations.
pub use clock::SystemClock;
pub use conflict_resolver::DefaultConflictResolver;
pub use errors::{ErrorConverter, InfrastructureError};
pub use filesystem::StdFileSystem;
pub use id_generator::UuidGenerator;
pub use repository::{
    JsonConfigurationRepository, JsonOperationRepository, JsonSettingsRepository,
};
