//! Application state managed by Tauri.
//!
//! This module defines the `AppState` struct that holds all long-lived
//! resources needed by Tauri commands.  It follows the pattern of
//! **dependency injection at startup**: all concrete services are
//! created in `main.rs` and stored here, so command handlers simply
//! extract them from the Tauri state.

use parking_lot::Mutex;
use quicksort_application::ApplicationFacadeImpl;
use quicksort_application::dtos::{OperationCommand, OperationResult};
use quicksort_application::errors::UseCaseError;

// OLD: use crate::folder::repository::JsonRepository;
// OLD: use crate::folder::service::FolderService;
// The old FolderService and related modules have been removed.
// All business logic now goes through the ApplicationFacade.

// OLD: LogEntry and logs – logging was previously done ad-hoc.
// Logging is now handled by domain events and the ActivityLog
// infrastructure adapter, so the state no longer needs to track logs.

/// Central state for the Tauri application.
///
/// Currently contains only the application facade, which provides
/// access to all use cases.  More fields (e.g., `exe_path`, `pipe_server`)
/// can be added as needed.
pub struct AppState {
    /// The unified entry point for all use cases.
    ///
    /// `ApplicationFacadeImpl` implements all inbound ports
    /// (`ExecuteOperation`, `UndoOperation`, `GetFolders`, `ManageFolders`)
    /// so commands only need a single reference.
    pub facade: ApplicationFacadeImpl,

    // Future fields (can be uncommented when needed):
    // pub exe_path: String,
    // pub admin_exe_path: String,
    // pub pending_file: Mutex<Option<String>>,
}

// OLD: LogEntry struct removed – logging is handled by infrastructure.