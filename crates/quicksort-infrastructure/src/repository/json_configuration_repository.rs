//! JSON-based implementation of ConfigurationRepository.
//! Stores folders in a JSON file at the given path.

use std::fs;
use std::path::PathBuf;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use quicksort_domain::{Folder, FolderId, FolderStats, WindowsPath};
use quicksort_application::ports::outbound::ConfigurationRepository;
use quicksort_application::errors::UseCaseError;

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
    sort_order: u32,
}

/// Repository that stores folder configuration in a JSON file.
pub struct JsonConfigurationRepository {
    path: PathBuf,
}

impl JsonConfigurationRepository {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    fn load_from_file(&self) -> Result<Vec<Folder>, UseCaseError> {
        if !self.path.exists() {
            return Ok(vec![]);
        }
        let content = fs::read_to_string(&self.path)
            .map_err(|e| UseCaseError::RepositoryError(e.to_string()))?;
        let config: ConfigFile = serde_json::from_str(&content)
            .map_err(|e| UseCaseError::RepositoryError(e.to_string()))?;

        // Convert each folder data to domain Folder, handling potential path validation errors.
        let mut folders = Vec::with_capacity(config.folders.len());
        for f in config.folders {
            let path = WindowsPath::new(&f.path)
                .map_err(|e| UseCaseError::RepositoryError(e.to_string()))?;
            let id = FolderId::from_string(&f.id)
                .map_err(|e| UseCaseError::RepositoryError(e.to_string()))?;
            folders.push(Folder::with_id(id, f.name, path));
        }
        Ok(folders)
    }

    fn save_to_file(&self, folders: &[Folder]) -> Result<(), UseCaseError> {
        let config = ConfigFile {
            version: 1,
            folders: folders.iter().map(|f| FolderData {
                id: f.id.to_string(),
                name: f.name.clone(),
                path: f.path.to_string(),
                is_favorite: f.favorite,
                sort_order: f.order,
            }).collect(),
        };
        let content = serde_json::to_string_pretty(&config)
            .map_err(|e| UseCaseError::RepositoryError(e.to_string()))?;
        fs::write(&self.path, content)
            .map_err(|e| UseCaseError::RepositoryError(e.to_string()))?;
        Ok(())
    }
}

#[async_trait]
impl ConfigurationRepository for JsonConfigurationRepository {
    async fn load_all(&self) -> Result<Vec<Folder>, UseCaseError> {
        self.load_from_file()
    }

    async fn save_all(&self, folders: &[Folder]) -> Result<(), UseCaseError> {
        self.save_to_file(folders)
    }

    async fn add(&self, folder: Folder) -> Result<(), UseCaseError> {
        let mut folders = self.load_from_file()?;
        folders.push(folder);
        self.save_to_file(&folders)
    }

    async fn remove(&self, id: &FolderId) -> Result<(), UseCaseError> {
        let mut folders = self.load_from_file()?;
        folders.retain(|f| f.id != *id);
        self.save_to_file(&folders)
    }

    async fn find_by_id(&self, id: &FolderId) -> Result<Option<Folder>, UseCaseError> {
        let folders = self.load_from_file()?;
        Ok(folders.into_iter().find(|f| f.id == *id))
    }

    async fn find_by_path(&self, path: &str) -> Result<Option<Folder>, UseCaseError> {
        let folders = self.load_from_file()?;
        Ok(folders.into_iter().find(|f| f.path.to_string() == path))
    }

    /// Returns the ID of the default "Documents" folder.
    /// If not found, creates a new FolderId for it.
    async fn get_default_folder_id(&self) -> Result<FolderId, UseCaseError> {
        // Look for an existing "Documents" folder by path
        let documents_path = WindowsPath::new("C:\\Users\\Public\\Documents")
            .map_err(|e| UseCaseError::RepositoryError(e.to_string()))?;
        
        if let Some(folder) = self.find_by_path(&documents_path.to_string()).await? {
            return Ok(folder.id);
        }

        // If not found, create a new ID (for use when first saving)
        Ok(FolderId::new())
    }
}