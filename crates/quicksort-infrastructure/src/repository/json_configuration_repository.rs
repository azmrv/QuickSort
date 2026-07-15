//! JSON-based implementation of ConfigurationRepository.
//! Stores folders in a JSON file at the given path.

use std::fs;
use std::path::PathBuf;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use quicksort_domain::{Folder, FolderId, WindowsPath};
use quicksort_application::ports::outbound::ConfigurationRepository;
use quicksort_infrastructure::errors::InfrastructureError;

#[derive(Serialize, Deserialize)]
struct ConfigFile {
    version: u32,
    folders: Vec<FolderData>,
}

#[derive(Serialize, Deserialize)]
struct FolderData {
    id: String,
    name: String,
    path: String,
    is_favorite: bool,
    sort_order: i32,
}

/// Repository that stores folder configuration in a JSON file.
pub struct JsonConfigurationRepository {
    path: PathBuf,
}

impl JsonConfigurationRepository {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    fn load_from_file(&self) -> Result<Vec<Folder>, InfrastructureError> {
        if !self.path.exists() {
            return Ok(vec![]);
        }
        let content = fs::read_to_string(&self.path)?;
        let config: ConfigFile = serde_json::from_str(&content)?;

        // Convert each folder data to domain Folder, handling potential path validation errors.
        let mut folders = Vec::with_capacity(config.folders.len());
        for f in config.folders {
            let path = WindowsPath::new(&f.path)
                .map_err(|e| InfrastructureError::Repository(e.to_string()))?;
            folders.push(Folder {
                id: FolderId::from_string(f.id),
                name: f.name,
                path,
                is_favorite: f.is_favorite,
                sort_order: f.sort_order,
            });
        }
        Ok(folders)
    }

    fn save_to_file(&self, folders: &[Folder]) -> Result<(), InfrastructureError> {
        let config = ConfigFile {
            version: 1,
            folders: folders.iter().map(|f| FolderData {
                id: f.id.as_str().to_string(),
                name: f.name.clone(),
                path: f.path.as_str().to_string(),
                is_favorite: f.is_favorite,
                sort_order: f.sort_order,
            }).collect(),
        };
        let content = serde_json::to_string_pretty(&config)?;
        fs::write(&self.path, content)
            .map_err(|e| InfrastructureError::Serialization(e.to_string()))?;
        Ok(())
    }
}

#[async_trait]
impl ConfigurationRepository for JsonConfigurationRepository {
    async fn load_all(&self) -> Result<Vec<Folder>, InfrastructureError> {
        self.load_from_file()
    }

    async fn save_all(&self, folders: &[Folder]) -> Result<(), InfrastructureError> {
        self.save_to_file(folders)
    }

    async fn add(&self, folder: Folder) -> Result<(), InfrastructureError> {
        let mut folders = self.load_from_file()?;
        folders.push(folder);
        self.save_to_file(&folders)
    }

    async fn remove(&self, id: &FolderId) -> Result<(), InfrastructureError> {
        let mut folders = self.load_from_file()?;
        folders.retain(|f| f.id != *id);
        self.save_to_file(&folders)
    }

    async fn find_by_id(&self, id: &FolderId) -> Result<Option<Folder>, InfrastructureError> {
        let folders = self.load_from_file()?;
        Ok(folders.into_iter().find(|f| f.id == *id))
    }

    async fn find_by_path(&self, path: &str) -> Result<Option<Folder>, InfrastructureError> {
        let folders = self.load_from_file()?;
        Ok(folders.into_iter().find(|f| f.path.as_str() == path))
    }

    /// Возвращает ID папки "Документы" (по умолчанию).
    /// Если папка не найдена, создаёт новый FolderId для неё.
    async fn get_default_folder_id(&self) -> Result<FolderId, InfrastructureError> {
        // Ищем существующую папку "Документы" по пути
        let documents_path = WindowsPath::new("C:\\Users\\Public\\Documents")?;
        
        if let Some(folder) = self.find_by_path(documents_path.as_str())? {
            return Ok(folder.id);
        }

        // Если не найдено, создаём новый ID (для использования при первой записи)
        Ok(FolderId::new())
    }
}
