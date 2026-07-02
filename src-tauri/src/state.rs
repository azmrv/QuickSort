use parking_lot::Mutex;
use crate::folder::repository::JsonRepository;
use crate::folder::service::FolderService;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LogEntry {
    pub timestamp: String,
    pub event: String,
    pub status: String,
}
pub struct AppState {
    pub service: FolderService<JsonRepository>,
    pub exe_path: Mutex<String>,
    pub admin_exe_path: String,
    pub logs: Mutex<Vec<LogEntry>>,
}