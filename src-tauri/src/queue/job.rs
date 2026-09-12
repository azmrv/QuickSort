//! Job model for the persistent operation queue.

use chrono::Utc;
use quicksort_application::OperationSource;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use quicksort_ipc_contract::{ExecuteOperationData, JobProgressDto, JobStatusDto};

/// Lifecycle state of a queued job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Canceled,
}

impl JobStatus {
    pub fn to_dto(self) -> JobStatusDto {
        match self {
            JobStatus::Queued => JobStatusDto::Queued,
            JobStatus::Running => JobStatusDto::Running,
            JobStatus::Completed => JobStatusDto::Completed,
            JobStatus::Failed => JobStatusDto::Failed,
            JobStatus::Canceled => JobStatusDto::Canceled,
        }
    }
}

/// A single entry in the persistent operation queue.
///
/// The raw `ExecuteOperationData` is stored so a job can be replayed after
/// an app restart without depending on a live `OperationCommand` in memory.
///
/// `source`/`correlation_id` live on the *job*, not on the wire contract:
/// the IPC contract is exclusively used by the context-menu DLL (source is
/// always `ContextMenu` there), while frontend enqueues go through Tauri
/// commands and must keep their origin across replay (spec #15, 0.2.6).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub data: ExecuteOperationData,
    #[serde(default)]
    pub source: OperationSource,
    #[serde(default)]
    pub correlation_id: Option<Uuid>,
    pub status: JobStatus,
    pub current: u64,
    pub total: u64,
    pub operation_id: Option<String>,
    pub error: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,
}

impl Job {
    pub fn new(
        id: String,
        data: ExecuteOperationData,
        source: OperationSource,
        correlation_id: Option<Uuid>,
    ) -> Self {
        let now = Utc::now().timestamp().max(0) as u64;
        let total = data.source_paths.len() as u64;
        Self {
            id,
            data,
            source,
            correlation_id,
            status: JobStatus::Queued,
            current: 0,
            total,
            operation_id: None,
            error: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn to_dto(&self) -> quicksort_ipc_contract::JobDto {
        quicksort_ipc_contract::JobDto {
            id: self.id.clone(),
            operation_type: self.data.operation_type.clone(),
            source_paths: self.data.source_paths.clone(),
            status: self.status.to_dto(),
            progress: JobProgressDto {
                current: self.current,
                total: self.total,
            },
            operation_id: self.operation_id.clone(),
            error: self.error.clone(),
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }

    pub fn touch(&mut self) {
        self.updated_at = Utc::now().timestamp().max(0) as u64;
    }
}
