//! Infrastructure implementations of outbound ports.

pub mod clock;
pub mod conflict_resolver;
pub mod duplicate_checker;
pub mod errors;
pub mod filesystem;
pub mod id_generator;
pub mod plugin;
pub mod repository;

// Re-export commonly used implementations.
pub use clock::SystemClock;
pub use conflict_resolver::DefaultConflictResolver;
pub use duplicate_checker::{ContentChecker, DuplicateDetectionAdapter, NameChecker, SizeChecker};
pub use errors::{ErrorConverter, InfrastructureError};
pub use filesystem::{FsFileSearch, StdFileSystem};
pub use id_generator::UuidGenerator;
// WcxPluginAdapter is Windows-only (WCX DLLs); re-export is gated
// to match the module declaration in plugin/mod.rs.
#[cfg(target_os = "windows")]
pub use plugin::{WcxPluginAdapter, WcxPluginLoader};
pub use repository::{
    JsonConfigurationRepository, JsonOperationRepository, JsonSettingsRepository,
};
