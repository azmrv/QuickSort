use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FolderId(pub Uuid);

impl Default for FolderId {
    fn default() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FolderStats {
    pub use_count: u64,
    pub last_used: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub version: u32,
    pub folders: Vec<Folder>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Folder {
    pub id: FolderId,
    pub name: String,
    pub path: PathBuf,
    #[serde(default)]
    pub favorite: bool,
    #[serde(default)]
    pub order: u32,
    #[serde(default)]
    pub stats: FolderStats,
}

