use anyhow::Result;
use crate::models::Config;

pub trait FolderRepository {
    fn load(&self) -> Result<Config>;
    fn save(&self, config: &Config) -> Result<()>;
}

pub struct JsonRepository {
    pub(in crate::folder) path: std::path::PathBuf,
}

impl JsonRepository {
    /// Создаёт новый репозиторий, хранящий данные в %APPDATA%\QuickSort\folders.json
    pub fn new() -> Result<Self> {
        let dir = std::env::var("APPDATA")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .join("QuickSort");
        std::fs::create_dir_all(&dir)?;
        Ok(Self {
            path: dir.join("folders.json"),
        })
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