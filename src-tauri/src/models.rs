//! Domain models for the Tauri adapter – **DEPRECATED**.
//!
//! These types were originally defined here during early development but
//! have since been moved to `quicksort-domain` and `quicksort-application`
//! as part of the Clean Architecture migration.
//!
//! This file is kept temporarily to avoid breaking existing imports during
//! the transition.  New code should use the canonical types:
//! - `quicksort_domain::Folder`
//! - `quicksort_domain::FolderId`
//! - `quicksort_domain::FolderStats`
//! - `quicksort_domain::OperationType` (etc.)
//! - `quicksort_application::OperationCommand`
//! - `quicksort_application::OperationResult`
//!
//! Once all references in this crate have been updated, this file should
//! be deleted.

// OLD: use std::path::PathBuf;
// OLD: use serde::{Deserialize, Serialize};
// OLD: use uuid::Uuid;
// OLD: use chrono::{DateTime, Utc};

// Re-export the canonical types so that existing imports continue to work.
// This is a temporary bridge; each `use crate::models::...` should be
// replaced with the proper domain or application import.
pub use quicksort_domain::{Folder, FolderId, FolderStats};
pub use quicksort_application::dtos::{OperationCommand as AppOperationCommand, OperationResult as AppOperationResult};
// `Config` is an application-level concern and is now in `quicksort_application`.
pub use quicksort_application::Config;

// OLD struct definitions are completely removed – they would conflict with
// the re-exports above.