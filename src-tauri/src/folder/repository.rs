use std::path::PathBuf;
use anyhow::Result;
use crate::models::Config;

pub trait FolderRepository {
    fn load(&self) -> Result<Config>;
    fn save(&self, config: &Config) -> Result<()>;
}

pub struct JsonRepository {
    pub(in crate::folder) path: PathBuf,
}

impl JsonRepository {
    pub fn new() -> Result<Self> {
        let appdata = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
        let mut path = PathBuf::from(appdata);
        path.push("QuickSort");
        std::fs::create_dir_all(&path)?;
        Ok(Self { path: path.join("folders.json") })
    }
}




impl FolderRepository for JsonRepository {
    fn load(&self) -> Result<Config> {
        if self.path.exists() {
            let data = std::fs::read_to_string(&self.path)?;
            Ok(serde_json::from_str(&data)?)
        } else {
            Ok(Config {
                version: 1,
                folders: vec![],
            })
        }
    }

    fn save(&self, config: &Config) -> Result<()> {
        let json = serde_json::to_string_pretty(config)?;
        std::fs::write(&self.path, json)?;
        Ok(())
    }
}