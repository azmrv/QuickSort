//! Persistent JSON-backed store for the operation queue.

use std::fs;
use std::path::PathBuf;

use super::job::Job;

/// Loads the job list from `queue.json`. A missing or corrupt file yields
/// an empty queue rather than failing app startup.
pub fn load(path: &PathBuf) -> Vec<Job> {
    match fs::read(path) {
        Ok(bytes) => serde_json::from_slice::<Vec<Job>>(&bytes).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

/// Persists the job list to `queue.json`. Best-effort: a failed write is
/// logged by the caller, not propagated to the job worker.
pub fn save(path: &PathBuf, jobs: &[Job]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let bytes = serde_json::to_vec_pretty(jobs)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    fs::write(path, bytes)
}
