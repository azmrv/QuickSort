//! Persistent operation queue (planner-worker-job).
//!
//! The queue accepts file operations from the IPC server and frontend,
//! persists them to `queue.json`, and executes them one at a time on a
//! dedicated worker thread. A single worker guarantees that history writes
//! from `ExecuteOperationUseCase` are never serialized concurrently.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use tauri::{AppHandle, Emitter};

use quicksort_application::{
    AbsolutePath, ApplicationFacadeImpl, DuplicateCheckMode, ExecuteOperation, FolderId,
    GetFolders, LoadSettings, OperationCommand, OperationType as DomainOpType,
    OverwritePolicy as AppOverwritePolicy,
};
use quicksort_ipc_contract::{
    DuplicateCheckMode as IpcDuplicateCheckMode, ExecuteOperationData, JobDto, JobId,
    OperationType as IpcOpType, OverwritePolicy as IpcOverwritePolicy,
};

pub mod job;
pub mod store;

use job::{Job, JobStatus};

/// Global Tauri AppHandle stored during setup, used to emit `job-status`
/// events to the frontend. Mirrors the handle pattern in `progress.rs`.
static APP_HANDLE: OnceLock<Mutex<Option<AppHandle>>> = OnceLock::new();

/// Stores the AppHandle so the queue can emit events. Called during setup.
pub fn set_app_handle(handle: AppHandle) {
    let _ = APP_HANDLE.set(Mutex::new(Some(handle)));
}

/// Resolves a `None` duplicate check mode to the mode configured in
/// settings.json (`duplicate_check.mode`), falling back to `Name`.
fn resolve_default_duplicate_check_mode(
    facade: &Arc<ApplicationFacadeImpl>,
    rt: &tokio::runtime::Runtime,
) -> IpcDuplicateCheckMode {
    let mode = match rt.block_on(facade.load_settings()) {
        Ok(settings) => settings.duplicate_check.mode,
        Err(e) => {
            tracing::warn!(error = %e, "settings unavailable; falling back to Name");
            return IpcDuplicateCheckMode::Name;
        }
    };
    // The settings DTO's DuplicateCheckMode is a distinct type from the
    // application's; both serialize to the same lowercase strings, so
    // convert through JSON to avoid a direct domain dependency.
    serde_json::from_value(serde_json::to_value(mode).unwrap_or_default()).unwrap_or_default()
}

/// Converts an IPC `ExecuteOperationData` into a domain `OperationCommand`,
/// mirroring the conversion in the IPC server. Returns `None` if the data
/// has no valid source paths.
pub fn convert_execute_data(data: &ExecuteOperationData) -> Option<OperationCommand> {
    let source_paths: Vec<AbsolutePath> = data
        .source_paths
        .iter()
        .filter_map(|p| AbsolutePath::new(p).ok())
        .collect();
    if source_paths.is_empty() {
        return None;
    }
    let target_folder_id = data
        .target_folder_id
        .as_ref()
        .and_then(|id| FolderId::from_string(id).ok());
    Some(OperationCommand {
        operation_type: match data.operation_type {
            IpcOpType::Move => DomainOpType::Move,
            IpcOpType::Copy => DomainOpType::Copy,
            IpcOpType::Delete => DomainOpType::Delete,
            IpcOpType::Rename => DomainOpType::Rename,
        },
        source_paths,
        target_folder_id,
        target_paths: None,
        overwrite_policy: match data.overwrite_policy {
            IpcOverwritePolicy::Skip => AppOverwritePolicy::Skip,
            IpcOverwritePolicy::Overwrite => AppOverwritePolicy::Overwrite,
            IpcOverwritePolicy::AutoRename => AppOverwritePolicy::AutoRename,
            IpcOverwritePolicy::Ask => AppOverwritePolicy::AutoRename,
            // Unreachable: Default is normalized to a concrete policy in the
            // IPC server before the queue ever sees it.
            IpcOverwritePolicy::Default => AppOverwritePolicy::Skip,
        },
        duplicate_check_mode: match data.duplicate_check_mode {
            // Unreachable: None is normalized to the configured mode in the
            // IPC server (and in run_job for replayed jobs) before conversion.
            Some(IpcDuplicateCheckMode::Name) | None => DuplicateCheckMode::Name,
            Some(IpcDuplicateCheckMode::Size) => DuplicateCheckMode::Size,
            Some(IpcDuplicateCheckMode::Content) => DuplicateCheckMode::Content,
        },
    })
}

/// Converts an application `OperationCommand` into IPC `ExecuteOperationData`
/// so the queue can persist and replay it. Mirrors `convert_execute_data`
/// in reverse; `target_paths` is intentionally dropped — the queue worker
/// only consumes Move/Copy/Delete jobs addressed by `target_folder_id`.
pub fn command_to_execute_data(command: &OperationCommand) -> ExecuteOperationData {
    let source_paths: Vec<String> = command
        .source_paths
        .iter()
        .filter_map(|p| p.as_str().map(ToString::to_string))
        .collect();
    let target_folder_id = command.target_folder_id.map(|id| id.to_string());
    ExecuteOperationData {
        operation_type: match command.operation_type {
            DomainOpType::Move => IpcOpType::Move,
            DomainOpType::Copy => IpcOpType::Copy,
            DomainOpType::Delete => IpcOpType::Delete,
            DomainOpType::Rename => IpcOpType::Rename,
        },
        source_paths,
        target_folder_id,
        target_folder_path: None,
        overwrite_policy: match command.overwrite_policy {
            AppOverwritePolicy::Skip => IpcOverwritePolicy::Skip,
            AppOverwritePolicy::Overwrite => IpcOverwritePolicy::Overwrite,
            AppOverwritePolicy::AutoRename => IpcOverwritePolicy::AutoRename,
            AppOverwritePolicy::Ask => IpcOverwritePolicy::Ask,
        },
        duplicate_check_mode: Some(match command.duplicate_check_mode {
            DuplicateCheckMode::Name => IpcDuplicateCheckMode::Name,
            DuplicateCheckMode::Size => IpcDuplicateCheckMode::Size,
            DuplicateCheckMode::Content => IpcDuplicateCheckMode::Content,
        }),
    }
}

/// Shared queue state plus the running worker flag.
pub struct JobQueue {
    state: Arc<Mutex<QueueState>>,
    store_path: PathBuf,
    op_lock: Arc<Mutex<()>>,
    worker_running: Arc<AtomicBool>,
}

struct QueueState {
    jobs: Vec<Job>,
    facade: Arc<ApplicationFacadeImpl>,
}

/// Payload emitted on the `job-status` Tauri event.
#[derive(Clone, serde::Serialize)]
pub struct JobStatusPayload {
    pub job: JobDto,
}

impl JobQueue {
    pub fn new(
        store_path: PathBuf,
        facade: Arc<ApplicationFacadeImpl>,
        op_lock: Arc<Mutex<()>>,
    ) -> Arc<Self> {
        let jobs = store::load(&store_path);
        Arc::new(Self {
            state: Arc::new(Mutex::new(QueueState { jobs, facade })),
            store_path,
            op_lock,
            worker_running: Arc::new(AtomicBool::new(false)),
        })
    }

    /// Starts the worker loop unless one is already running.
    pub fn start_worker(self: &Arc<Self>) {
        if self.worker_running.swap(true, Ordering::SeqCst) {
            return;
        }
        let this = Arc::clone(self);
        std::thread::Builder::new()
            .name("queue-worker".into())
            .spawn(move || this.worker_loop())
            .expect("failed to spawn queue worker thread");
    }

    fn persist(&self, state: &QueueState) {
        if let Err(e) = store::save(&self.store_path, &state.jobs) {
            tracing::error!(error = %e, "queue persist failed");
        }
    }

    fn emit_status(&self, job: &Job) {
        if let Some(handle) = APP_HANDLE.get() {
            if let Ok(guard) = handle.lock() {
                if let Some(ref h) = *guard {
                    let _ = h.emit("job-status", JobStatusPayload { job: job.to_dto() });
                }
            }
        }
    }

    /// Adds a new job to the tail of the queue and wakes the worker.
    pub fn enqueue(&self, data: ExecuteOperationData) -> Result<JobId, String> {
        if convert_execute_data(&data).is_none() {
            return Err("Invalid command: no valid source paths".to_string());
        }
        let id = uuid::Uuid::new_v4().to_string();
        let job = Job::new(id.clone(), data);

        let mut guard = self.state.lock().unwrap_or_else(|e| e.into_inner());
        guard.jobs.push(job.clone());
        self.persist(&guard);

        drop(guard);

        tracing::info!(job_id = %id, "queued operation");
        self.emit_status(&job);
        Ok(JobId { id })
    }

    /// Marks a queued (not yet started) job as Canceled.
    pub fn cancel(&self, id: &str) -> Result<(), String> {
        let snapshot = {
            let mut guard = self.state.lock().unwrap_or_else(|e| e.into_inner());
            let job = guard
                .jobs
                .iter_mut()
                .find(|j| j.id == id)
                .ok_or_else(|| "Job not found".to_string())?;
            if job.status != JobStatus::Queued {
                return Err("Only queued jobs can be canceled".to_string());
            }
            job.status = JobStatus::Canceled;
            job.touch();
            let snapshot = job.clone();
            self.persist(&guard);
            snapshot
        };
        self.emit_status(&snapshot);
        Ok(())
    }

    /// Serializes the current job list for the frontend.
    pub fn list(&self) -> Vec<JobDto> {
        let guard = self.state.lock().unwrap_or_else(|e| e.into_inner());
        guard.jobs.iter().map(|j| j.to_dto()).collect()
    }

    /// Serializes a single job, if present.
    pub fn get(&self, id: &str) -> Option<JobDto> {
        let guard = self.state.lock().unwrap_or_else(|e| e.into_inner());
        guard.jobs.iter().find(|j| j.id == id).map(|j| j.to_dto())
    }

    fn worker_loop(self: &Arc<Self>) {
        tracing::info!("queue worker started");
        loop {
            let next = {
                let mut guard = self.state.lock().unwrap_or_else(|e| e.into_inner());
                let position = guard
                    .jobs
                    .iter()
                    .position(|j| j.status == JobStatus::Queued);
                match position {
                    Some(idx) => {
                        let snapshot = {
                            let job = &mut guard.jobs[idx];
                            job.status = JobStatus::Running;
                            job.touch();
                            job.clone()
                        };
                        self.persist(&guard);
                        drop(guard);
                        self.emit_status(&snapshot);
                        Some(snapshot)
                    }
                    None => {
                        drop(guard);
                        None
                    }
                }
            };

            match next {
                Some(job) => self.run_job(job),
                None => std::thread::sleep(std::time::Duration::from_millis(300)),
            }
        }
    }

    fn run_job(&self, mut job: Job) {
        // Serialize against the IPC server's background operations so the
        // shared JSON history repository is never written concurrently.
        let _guard = self.op_lock.lock().unwrap_or_else(|e| e.into_inner());

        let registry_facade = self
            .state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .facade
            .clone();

        let worker_rt = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(rt) => rt,
            Err(e) => {
                tracing::error!(job_id = %job.id, error = %e, "failed to create worker runtime");
                job.status = JobStatus::Failed;
                job.error = Some(format!("Runtime error: {}", e));
                job.touch();
                self.update_job(job);
                return;
            }
        };

        // Resolve a None duplicate check mode to the user-configured default
        // (settings.json) so replayed jobs honor the configured mode.
        if job.data.duplicate_check_mode.is_none() {
            job.data.duplicate_check_mode = Some(resolve_default_duplicate_check_mode(
                &registry_facade,
                &worker_rt,
            ));
        }

        let command = match convert_execute_data(&job.data) {
            Some(cmd) => cmd,
            None => {
                tracing::error!(job_id = %job.id, "job has no valid command");
                job.status = JobStatus::Failed;
                job.error = Some("Invalid command: no valid source paths".to_string());
                job.touch();
                self.update_job(job);
                return;
            }
        };

        let mut command = command;
        if command.target_folder_id.is_none() {
            if let Some(raw_path) = job.data.target_folder_path.clone() {
                let folders = worker_rt
                    .block_on(registry_facade.get_all())
                    .unwrap_or_default();
                command.target_folder_id = folders
                    .iter()
                    .find(|f| f.path.to_string() == raw_path)
                    .map(|f| f.id);
                if command.target_folder_id.is_none() {
                    tracing::error!(job_id = %job.id, path = %raw_path, "target folder not found");
                    job.status = JobStatus::Failed;
                    job.error = Some(format!("Target folder not found: {}", raw_path));
                    job.touch();
                    self.update_job(job);
                    return;
                }
            }
        }

        tracing::info!(
            job_id = %job.id,
            op_type = ?command.operation_type,
            files = command.source_paths.len(),
            "job started"
        );
        match worker_rt.block_on(registry_facade.execute(command)) {
            Ok(result) => {
                job.status = JobStatus::Completed;
                job.current = job.total;
                job.operation_id = Some(result.operation_id.to_string());
                job.touch();
                tracing::info!(
                    job_id = %job.id,
                    op_id = ?job.operation_id,
                    files = result.processed_files,
                    "job completed"
                );
            }
            Err(e) => {
                job.status = JobStatus::Failed;
                job.error = Some(e.to_string());
                job.touch();
                tracing::error!(job_id = %job.id, error = %e, "job failed");
            }
        }
        self.update_job(job);
    }

    fn update_job(&self, job: Job) {
        let mut guard = self.state.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(existing) = guard.jobs.iter_mut().find(|j| j.id == job.id) {
            *existing = job.clone();
        }
        self.persist(&guard);
        drop(guard);
        self.emit_status(&job);
    }
}
