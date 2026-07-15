//! Outbound port for generating unique identifiers.
//!
//! This port provides a way for the Application and Domain layers to
//! obtain unique IDs without depending on a specific ID generation
//! strategy (UUID v4, UUID v7, ULID, etc.).
//!
//! # Design Decision
//! The port returns `quicksort_domain::OperationId` rather than a raw
//! `String` to maintain type safety and domain invariants. The domain
//! defines `OperationId` as a newtype wrapper, ensuring that only
//! valid identifiers circulate through the system.

use quicksort_domain::OperationId;

/// Port for generating unique operation identifiers.
///
/// The Infrastructure layer implements this port using a concrete
/// generator (e.g., `UuidGenerator` that delegates to `uuid::Uuid::new_v4()`).
/// The Application layer depends only on this trait, making it easy
/// to switch ID generation strategies or mock the generator in tests.
pub trait IdGenerator: Send + Sync {
    /// Generates a new unique identifier.
    ///
    /// # Returns
    /// An `OperationId` wrapping a unique value.
    ///
    /// # Usage
    /// Called by Use Cases when creating new `Operation` aggregates.
    fn generate(&self) -> OperationId;
}

// OLD: used String
// pub trait IdGenerator: Send + Sync {
//     fn generate(&self) -> String;
// }
// NEW: switched to `OperationId` for type safety and domain alignment