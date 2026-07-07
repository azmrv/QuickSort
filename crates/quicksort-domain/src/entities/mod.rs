//! Domain entities – aggregates and business objects.

use crate::value_objects::{FolderId, OperationId, WindowsPath};
use crate::events::DomainEvent;
use crate::errors::DomainError;
use std::time::SystemTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Folder {
    pub id: FolderId,
    pub name: String,
    pub path: WindowsPath,
    pub is_favorite: bool,
    pub sort_order: i32,
}

impl Folder {
    pub fn new(id: FolderId, name: String, path: WindowsPath) -> Self {
        Self {
            id,
            name,
            path,
            is_favorite: false,
            sort_order: 0,
        }
    }
}

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

#[derive(Debug, Clone)]
pub struct Operation {
    pub id: OperationId,
    pub operation_type: OperationType,
    pub state: OperationState,
    pub source_paths: Vec<WindowsPath>,
    pub target_folder_path: Option<WindowsPath>,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
    events: Vec<DomainEvent>,
}

impl Operation {
    pub fn new(
        id: OperationId,
        op_type: OperationType,
        source_paths: Vec<WindowsPath>,
        target: Option<WindowsPath>,
        now: SystemTime,
    ) -> Self {
        Self {
            id,
            operation_type: op_type,
            state: OperationState::Pending,
            source_paths,
            target_folder_path: target,
            created_at: now,
            updated_at: now,
            events: Vec::new(),
        }
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
}