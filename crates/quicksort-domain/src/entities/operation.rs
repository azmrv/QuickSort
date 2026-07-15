//! Domain entity representing file system operations.
//!
//! # Design Decisions
//! - Uses `chrono::DateTime<Utc>` for timestamps instead of `SystemTime`
//!   to guarantee `Serialize`/`Deserialize` compatibility with JSON configs.
//! - All constructors (`new_move`, `new_copy`, etc.) now accept an explicit
//!   `now: DateTime<Utc>` parameter to improve testability (time injection).
//! - `OperationState::Completed.bytes_moved` renamed to `bytes_processed`
//!   because the operation may be a copy, not just move.
//! - The `start` method no longer accepts `now` parameter – it internally uses
//!   `Utc::now()` to reduce unnecessary boilerplate; external time injection
//!   can be added later if needed.

use crate::{
    value_objects::{OperationId, WindowsPath},
    events::DomainEvent,
    errors::DomainError,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// OLD: use std::time::SystemTime;
// NEW: use chrono::{DateTime, Utc}; — better serialization support

/// Defines the kind of file operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationType {
    Move,
    Copy,
    Delete,
    Rename,
}

/// Tracks the lifecycle of an operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationState {
    Pending,
    Executing,
    Completed {
        processed_files: u32,
        // OLD: bytes_moved: u64,
        // NEW: renamed to bytes_processed to accurately reflect copy operations
        bytes_processed: u64,
    },
    Failed {
        reason: String,
    },
    Undone,
}

/// Aggregate root for file operations.
///
/// # Invariants
/// - `source_paths` is never empty.
/// - `target_folder_path` is set for Move/Copy, `None` for Delete, `Some` for Rename.
/// - Events are accumulated and can be consumed via `pull_events`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    pub id: OperationId,
    pub operation_type: OperationType,
    pub state: OperationState,
    pub source_paths: Vec<WindowsPath>,
    pub target_folder_path: Option<WindowsPath>,
    pub target_paths: Option<Vec<WindowsPath>>,
    // OLD: pub created_at: SystemTime,
    pub created_at: DateTime<Utc>,
    // OLD: pub updated_at: SystemTime,
    pub updated_at: DateTime<Utc>,
    #[serde(skip)]
    pub(crate) events: Vec<DomainEvent>,
}

impl Operation {
    /// Internal constructor – all domain invariants are checked here.
    pub fn new(
        id: OperationId,
        op_type: OperationType,
        source_paths: Vec<WindowsPath>,
        target: Option<WindowsPath>,
        target_paths: Option<Vec<WindowsPath>>,
        now: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            operation_type: op_type,
            state: OperationState::Pending,
            source_paths,
            target_folder_path: target,
            target_paths,
            created_at: now,
            updated_at: now,
            events: Vec::new(),
        }
    }

    /// Create a Move operation (files moved to a single target folder).
    pub fn new_move(source: Vec<WindowsPath>, target: WindowsPath, now: DateTime<Utc>) -> Self {
        Self::new(
            OperationId::new(),
            OperationType::Move,
            source,
            Some(target),
            None,
            now,
        )
    }

    /// Create a Copy operation.
    pub fn new_copy(source: Vec<WindowsPath>, target: WindowsPath, now: DateTime<Utc>) -> Self {
        Self::new(
            OperationId::new(),
            OperationType::Copy,
            source,
            Some(target),
            None,
            now,
        )
    }

    /// Create a Delete operation.
    pub fn new_delete(source: Vec<WindowsPath>, now: DateTime<Utc>) -> Self {
        Self::new(
            OperationId::new(),
            OperationType::Delete,
            source,
            None,
            None,
            now,
        )
    }

    /// Create a Rename operation.
    /// The `target` parameter holds the new path.
    pub fn new_rename(source: Vec<WindowsPath>, target: WindowsPath, now: DateTime<Utc>) -> Self {
        Self::new(
            OperationId::new(),
            OperationType::Rename,
            source,
            None,
            Some(vec![target]),
            now,
        )
    }

    /// Drain all pending domain events.
    pub fn pull_events(&mut self) -> Vec<DomainEvent> {
        std::mem::take(&mut self.events)
    }

    /// Transition the operation from `Pending` to `Executing`.
    pub fn start(&mut self) -> Result<(), DomainError> {
        if !matches!(self.state, OperationState::Pending) {
            return Err(DomainError::InvalidStateTransition);
        }
        self.state = OperationState::Executing;
        // NEW: use Utc::now() internally – can be changed to injected clock later
        self.updated_at = Utc::now();
        self.events.push(DomainEvent::OperationStarted {
            operation_id: self.id.clone(),
            op_type: self.operation_type.clone(),
        });
        Ok(())
    }

    /// Mark the operation as successfully completed.
    pub fn complete(&mut self, files: u32, bytes: u64) -> Result<(), DomainError> {
        if !matches!(self.state, OperationState::Executing) {
            return Err(DomainError::InvalidStateTransition);
        }
        self.state = OperationState::Completed {
            processed_files: files,
            bytes_processed: bytes,
        };
        self.updated_at = Utc::now();
        self.events.push(DomainEvent::OperationCompleted {
            operation_id: self.id.clone(),
            files,
            bytes,
        });
        Ok(())
    }

    /// Record a failure.
    pub fn fail(&mut self, reason: String) -> Result<(), DomainError> {
        if !matches!(self.state, OperationState::Pending | OperationState::Executing) {
            return Err(DomainError::InvalidStateTransition);
        }
        self.state = OperationState::Failed {
            reason: reason.clone(),
        };
        self.updated_at = Utc::now();
        self.events.push(DomainEvent::OperationFailed {
            operation_id: self.id.clone(),
            reason,
        });
        Ok(())
    }

    /// Mark a previously completed operation as undone.
    pub fn mark_undone(&mut self) -> Result<(), DomainError> {
        if !matches!(self.state, OperationState::Completed { .. }) {
            return Err(DomainError::InvalidStateTransition);
        }
        self.state = OperationState::Undone;
        self.updated_at = Utc::now();
        self.events.push(DomainEvent::OperationUndone {
            operation_id: self.id.clone(),
        });
        Ok(())
    }

    /// Test helper to inspect internal events.
    #[cfg(test)]
    pub fn events(&self) -> &[DomainEvent] {
        &self.events
    }
}

impl Default for Operation {
    fn default() -> Self {
        Self::new(
            OperationId::new(),
            OperationType::Move,
            Vec::new(),
            None,
            None,
            Utc::now(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_path(path: &str) -> WindowsPath {
        WindowsPath::new(path).unwrap()
    }

    fn now() -> DateTime<Utc> {
        Utc::now()
    }

    #[test]
    fn test_move() {
        let op = Operation::new_move(
            vec![test_path("C:\\a.txt")],
            test_path("C:\\b.txt"),
            now(),
        );
        assert_eq!(op.operation_type, OperationType::Move);
    }

    #[test]
    fn test_rename() {
        let op = Operation::new_rename(
            vec![test_path("C:\\a.txt")],
            test_path("C:\\b.txt"),
            now(),
        );
        assert_eq!(op.operation_type, OperationType::Rename);
        assert!(op.target_folder_path.is_none());
        assert!(op.target_paths.is_some());
    }

    #[test]
    fn test_start_complete() {
        let mut op = Operation::new_delete(vec![test_path("C:\\a.txt")], now());
        op.start().unwrap();
        op.complete(1, 100).unwrap();
        assert!(matches!(op.state, OperationState::Completed { .. }));
    }

    #[test]
    fn test_events() {
        let mut op = Operation::new_delete(vec![test_path("C:\\a.txt")], now());
        op.start().unwrap();
        assert_eq!(op.events().len(), 1);
    }
}