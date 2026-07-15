//! Outbound port for obtaining the current time.
//!
//! This port is implemented by the Infrastructure layer and injected
//! into Use Cases that need to record timestamps for operations,
//! domain events, or audit records.
//!
//! # Design Decision
//! We use `chrono::DateTime<Utc>` instead of `std::time::SystemTime`
//! because `DateTime<Utc>` implements `Serialize` and `Deserialize`
//! (via the `serde` feature of the `chrono` crate), which is required
//! for JSON-based operation persistence.
//! `SystemTime` does not implement these traits by default and would
//! require custom serialization logic, adding unnecessary complexity.

use chrono::{DateTime, Utc};

/// Port for obtaining the current timestamp.
///
/// The Infrastructure layer provides the actual clock implementation
/// (e.g., `SystemClock` that delegates to `Utc::now()`).
/// The Application and Domain layers depend only on this trait,
/// making time-related logic testable by injecting a mock clock.
pub trait Clock: Send + Sync {
    /// Returns the current date and time in UTC.
    ///
    /// # Returns
    /// A `DateTime<Utc>` representing the current moment.
    ///
    /// # Usage
    /// Used by Use Cases to set `created_at` and `updated_at` fields
    /// on domain entities and by the `Operation` aggregate to record
    /// state transition times.
    fn now(&self) -> DateTime<Utc>;
}

// OLD: used `std::time::SystemTime`
// pub trait Clock: Send + Sync {
//     fn now(&self) -> SystemTime;
// }
// NEW: switched to `chrono::DateTime<Utc>` for JSON compatibility