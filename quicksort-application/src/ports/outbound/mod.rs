// quicksort-application/src/ports/outbound/mod.rs

pub mod clock;                     // Clock – current time
pub mod configuration_repository;   // ConfigurationRepository – folder CRUD
pub mod file_system;               // FileSystem – file I/O operations
pub mod id_generator;              // IdGenerator – unique identifier generation
pub mod conflict_resolver;         // ConflictResolver – interactive conflict resolution

// Re-export the main interface for external consumers (Facade)
pub use clock::Clock;
pub use configuration_repository::ConfigurationRepository;
pub use file_system::FileSystem;
pub use id_generator::IdGenerator;
pub use conflict_resolver::ConflictResolver;