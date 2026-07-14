//! Domain entity representing file system operations.

use crate::{value_objects::{OperationId, WindowsPath}, events::DomainEvent, errors::DomainError};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationType {
    Move,
    Copy,
    Delete,
    Rename,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationState {
    Pending,
    Executing,
    Completed { processed_files: u32, bytes_moved: u64 },
    Failed { reason: String },
    Undone,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    pub id: OperationId,
    pub operation_type: OperationType,
    pub state: OperationState,
    pub source_paths: Vec<WindowsPath>,
    pub target_folder_path: Option<WindowsPath>,
    pub target_paths: Option<Vec<WindowsPath>>,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
    #[serde(skip)]
    pub(crate) events: Vec<DomainEvent>,
}

impl Operation {
    pub fn new(
        id: OperationId,
        op_type: OperationType,
        source_paths: Vec<WindowsPath>,
        target: Option<WindowsPath>,
        target_paths: Option<Vec<WindowsPath>>,
        now: SystemTime,
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

    pub fn new_move(source: Vec<WindowsPath>, target: WindowsPath) -> Self {
        Self::new(
            OperationId::new(),
            OperationType::Move,
            source,
            Some(target),
            None,
            SystemTime::now(),
        )
    }

    pub fn new_copy(source: Vec<WindowsPath>, target: WindowsPath) -> Self {
        Self::new(
            OperationId::new(),
            OperationType::Copy,
            source,
            Some(target),
            None,
            SystemTime::now(),
        )
    }

    pub fn new_delete(source: Vec<WindowsPath>) -> Self {
        Self::new(
            OperationId::new(),
            OperationType::Delete,
            source,
            None,
            None,
            SystemTime::now(),
        )
    }

    pub fn new_rename(source: Vec<WindowsPath>, target: WindowsPath, now: SystemTime) -> Self {
        Self::new(
            OperationId::new(),
            OperationType::Rename,
            source,
            None,
            Some(vec![target]),
            now,
        )
    }

    pub fn pull_events(&mut self) -> Vec<DomainEvent> {
        std::mem::take(&mut self.events)
    }

    pub fn start(&mut self, now: SystemTime) -> Result<(), DomainError> {
        if !matches!(self.state, OperationState::Pending) {
            return Err(DomainError::InvalidStateTransition);
        }
        self.state = OperationState::Executing;
        self.updated_at = now;
        self.events.push(DomainEvent::OperationStarted {
            operation_id: self.id.clone(),
            op_type: self.operation_type.clone(),
        });
        Ok(())
    }

    pub fn complete(&mut self, files: u32, bytes: u64, now: SystemTime) -> Result<(), DomainError> {
        if !matches!(self.state, OperationState::Executing) {
            return Err(DomainError::InvalidStateTransition);
        }
        self.state = OperationState::Completed { processed_files: files, bytes_moved: bytes };
        self.updated_at = now;
        self.events.push(DomainEvent::OperationCompleted {
            operation_id: self.id.clone(),
            files,
            bytes,
        });
        Ok(())
    }

    pub fn fail(&mut self, reason: String, now: SystemTime) -> Result<(), DomainError> {
        if !matches!(self.state, OperationState::Pending | OperationState::Executing) {
            return Err(DomainError::InvalidStateTransition);
        }
        self.state = OperationState::Failed { reason: reason.clone() };
        self.updated_at = now;
        self.events.push(DomainEvent::OperationFailed {
            operation_id: self.id.clone(),
            reason,
        });
        Ok(())
    }

    pub fn mark_undone(&mut self, now: SystemTime) -> Result<(), DomainError> {
        if !matches!(self.state, OperationState::Completed { .. }) {
            return Err(DomainError::InvalidStateTransition);
        }
        self.state = OperationState::Undone;
        self.updated_at = now;
        self.events.push(DomainEvent::OperationUndone {
            operation_id: self.id.clone(),
        });
        Ok(())
    }

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
            SystemTime::now(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_path(path: &str) -> WindowsPath {
        WindowsPath::new(path).unwrap()
    }

    #[test]
    fn test_move() {
        let op = Operation::new_move(vec![test_path("C:\\a.txt")], test_path("C:\\b.txt"));
        assert_eq!(op.operation_type, OperationType::Move);
    }

    #[test]
    fn test_rename() {
        let op = Operation::new_rename(vec![test_path("C:\\a.txt")], test_path("C:\\b.txt"), SystemTime::now());
        assert_eq!(op.operation_type, OperationType::Rename);
        assert!(op.target_folder_path.is_none());
        assert!(op.target_paths.is_some());
    }

    #[test]
    fn test_start_complete() {
        let mut op = Operation::new_delete(vec![test_path("C:\\a.txt")]);
        op.start(SystemTime::now()).unwrap();
        op.complete(1, 100, SystemTime::now()).unwrap();
        assert!(matches!(op.state, OperationState::Completed { .. }));
    }

    #[test]
    fn test_events() {
        let mut op = Operation::new_delete(vec![test_path("C:\\a.txt")]);
        op.start(SystemTime::now()).unwrap();
        assert_eq!(op.events().len(), 1);
    }
}