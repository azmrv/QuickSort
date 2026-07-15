//! Domain events – immutable records of business facts.
//!
//! Events are raised by domain aggregates when a significant state change
//! occurs.  They are consumed by the Application layer for logging, auditing,
//! and potentially by event-driven infrastructure (e.g., Event Sourcing).
//!
//! # Design Decision
//! Events are serializable so they can be persisted alongside operations,
//! but the primary mechanism for reading them is the `pull_events` method
//! on the `Operation` aggregate.  This keeps the domain free of messaging
//! infrastructure.

use serde::{Serialize, Deserialize};
use crate::value_objects::{OperationId, FolderId};
use crate::entities::OperationType;

/// Enumeration of all domain events in the QuickSort system.
///
/// Each variant carries the minimum data necessary to describe what happened,
/// allowing consumers to react or record the event without needing access
/// to the full aggregate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DomainEvent {
    /// An operation has started executing.
    OperationStarted {
        /// The unique ID of the operation.
        operation_id: OperationId,
        /// The type of operation (Move, Copy, Delete, Rename).
        op_type: OperationType,
    },

    /// An operation has successfully completed.
    OperationCompleted {
        /// The unique ID of the operation.
        operation_id: OperationId,
        /// Number of files processed.
        files: u32,
        /// Total bytes processed (moved, copied, or deleted).
        bytes: u64,
    },

    /// An operation has failed.
    OperationFailed {
        /// The unique ID of the operation.
        operation_id: OperationId,
        /// Human-readable description of the failure.
        reason: String,
    },

    /// A previously completed operation has been undone.
    OperationUndone {
        /// The unique ID of the operation that was undone.
        operation_id: OperationId,
    },

    /// A new folder has been added to the configuration.
    FolderAdded {
        /// The unique ID of the added folder.
        folder_id: FolderId,
        /// Display name of the folder.
        name: String,
        /// Absolute filesystem path of the folder.
        path: String,
    },

    /// A folder has been removed from the configuration.
    FolderRemoved {
        /// The unique ID of the removed folder.
        folder_id: FolderId,
        /// Display name of the folder at the time of removal.
        name: String,
    },

    /// A folder has been renamed.
    FolderRenamed {
        /// The unique ID of the renamed folder.
        folder_id: FolderId,
        /// Previous display name.
        old_name: String,
        /// New display name.
        new_name: String,
    },
}