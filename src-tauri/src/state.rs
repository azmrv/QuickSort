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
    pub logs: Mutex<Vec<LogEntry>>,
    // pub facade: ()
}