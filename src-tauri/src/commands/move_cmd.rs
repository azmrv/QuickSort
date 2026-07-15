//! Tauri command for executing file operations (Move, Copy, Delete, Rename).
//!
//! This command is called by the React frontend (SelectorPage) when the user
//! picks a folder from the "📂 Все папки..." dialog.  It delegates to the
//! Application Facade, which in turn calls `ExecuteOperationUseCase` and
//! persists the operation history.
//!
//! # Migration Note
//! The old `move_file` command that called `MoveEngine` directly has been
//! replaced by `execute_operation`, which accepts a full `OperationCommand`.
//! This allows the frontend to trigger any operation type, not just Move.

use tauri::State;
use crate::state::AppState;
use quicksort_application::dtos::{OperationCommand, OverwritePolicy};
use quicksort_domain::OperationType;

/// Executes a file operation (Move, Copy, Delete, or Rename).
///
/// The frontend constructs an `OperationCommand` with the appropriate
/// parameters and sends it here.  The command is forwarded to the
/// `ApplicationFacade`, which runs it through the full pipeline
/// (validation, execution, logging).
///
/// # Returns
/// A human-readable summary of the operation result, or an error message.
#[tauri::command]
pub async fn execute_operation(
    state: State<'_, AppState>,
    command: OperationCommand,
) -> Result<String, String> {
    // Delegate to the Application Facade – the single entry point for all
    // business operations.
    let result = state
        .facade
        .execute_operation(command)
        .await
        .map_err(|e| e.to_string())?;

    Ok(format!(
        "Operation {} completed: {} file(s), {} bytes processed",
        result.operation_id.to_string(),
        result.processed_files,
        result.bytes_moved,
    ))
}

// The old `move_file` function has been removed.  It:
// - Called the deprecated `MoveEngine` directly.
// - Logged the result twice (once into `state.logs` and once via
//   `activity_log`).
// - Returned a plain path string.
//
// All of this is now handled by `ExecuteOperationUseCase` and the
// domain events it raises.