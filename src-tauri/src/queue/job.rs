//! Job model for the persistent operation queue.

use chrono::Utc;
use serde::{Deserialize, Serialize};

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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub data: ExecuteOperationData,
    pub status: JobStatus,
    pub current: u32,
    pub total: u32,
    pub operation_id: Option<String>,
    pub error: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,
}

impl Job {
    pub fn new(id: String, data: ExecuteOperationData) -> Self {
        let now = Utc::now().timestamp().max(0) as u64;
        let total = data.source_paths.len() as u32;
        Self {
            id,
            data,
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
