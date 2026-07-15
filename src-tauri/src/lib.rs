//! Tauri adapter library – **transitional**.
//!
//! This file exposes the modules that are shared between the Tauri binary
//! (`main.rs`) and the integration tests.  During the Clean Architecture
//! migration, several modules were moved to dedicated crates:
//!
//! | Old module         | New location                                  |
//! |--------------------|-----------------------------------------------|
//! | `activity_log`     | Replaced by `JsonOperationRepository` + Domain Events |
//! | `folder`           | Replaced by `quicksort-application` ports     |
//! | `models`           | Moved to `quicksort-domain`                   |
//! | `move_engine`      | Moved to `quicksort-infrastructure::StdFileSystem` |
//!
//! The old modules have been removed from this file.  New modules (`state`,
//! `pending`, `logging`) are kept here because they are tightly coupled to
//! the Tauri adapter lifecycle and are not shared with other crates.

// Keep for now – provides `AppState` (will be replaced by Application Facade)
pub mod state;

// Keep – CLI `--select-folder` handler (will be moved into AppState later)
pub mod pending;

// Keep – logging initialisation (called by main.rs)
pub mod logging;