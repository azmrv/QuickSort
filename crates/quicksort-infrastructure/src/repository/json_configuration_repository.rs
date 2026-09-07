//! JSON-based implementation of ConfigurationRepository.
//! Stores folders in a JSON file at the given path.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use quicksort_application::errors::UseCaseError;
use quicksort_application::ports::outbound::ConfigurationRepository;
use quicksort_domain::{AbsolutePath, Folder, FolderId};

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
    #[serde(default)]
    favorite: bool,
    #[serde(default)]
    order: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    color: Option<String>,
    #[serde(default)]
    stats: Option<serde_json::Value>,
}

/// Repository that stores folder configuration in a JSON file.
pub struct JsonConfigurationRepository {
    path: PathBuf,
}

/// Returns a normalized key for a folder path, used to detect duplicates.
///
/// # Normalization rules
/// - Trailing path separators are stripped (`C:\Foo\` and `C:\Foo` are equal).
/// - On Windows the comparison is case-insensitive (`c:\foo` == `C:\FOO`).
fn path_key(path: &str) -> String {
    let trimmed = path.trim_end_matches(['\\', '/']);
    #[cfg(target_os = "windows")]
    {
        trimmed.to_lowercase()
    }
    #[cfg(not(target_os = "windows"))]
    {
        trimmed.to_string()
    }
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
            let path = AbsolutePath::new(&f.path)
                .map_err(|e| UseCaseError::RepositoryError(e.to_string()))?;
            let id = FolderId::from_string(&f.id)
                .map_err(|e| UseCaseError::RepositoryError(e.to_string()))?;
            let mut folder = Folder::with_id(id, f.name, path);
            if f.favorite {
                folder.toggle_favorite();
            }
            folder.order = f.order;
            folder.color = f.color;
            folders.push(folder);
        }

        // Dedup on load heals configs that accumulated duplicates before dedup existed.
        let mut seen: HashSet<String> = HashSet::new();
        folders.retain(|f| seen.insert(path_key(&f.path.to_string())));

        Ok(folders)
    }

    fn save_to_file(&self, folders: &[Folder]) -> Result<(), UseCaseError> {
        let config = ConfigFile {
            version: 1,
            folders: folders
                .iter()
                .map(|f| FolderData {
                    id: f.id.to_string(),
                    name: f.name.clone(),
                    path: f.path.to_string(),
                    favorite: f.favorite,
                    order: f.order,
                    color: f.color.clone(),
                    stats: None,
                })
                .collect(),
        };
        let content = serde_json::to_string_pretty(&config)
            .map_err(|e| UseCaseError::RepositoryError(e.to_string()))?;
        fs::write(&self.path, content).map_err(|e| UseCaseError::RepositoryError(e.to_string()))?;
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
        let key = path_key(&folder.path.to_string());
        if folders.iter().any(|f| path_key(&f.path.to_string()) == key) {
            return Err(UseCaseError::RepositoryError(format!(
                "Folder already exists: {}",
                folder.path
            )));
        }
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
        // On Windows, look for an existing "Documents" folder by path.
        #[cfg(target_os = "windows")]
        {
            let documents_path = AbsolutePath::new("C:\\Users\\Public\\Documents")
                .map_err(|e| UseCaseError::RepositoryError(e.to_string()))?;

            if let Some(folder) = self.find_by_path(&documents_path.to_string()).await? {
                return Ok(folder.id);
            }
        }

        // If not found, create a new ID (for use when first saving)
        Ok(FolderId::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn test_path(path: &str) -> AbsolutePath {
        // Same pattern as the domain tests: keep Windows fixture paths as-is
        // on Windows, convert drive-letter fixtures to Unix root-relative
        // paths so the same tests run on any platform.
        let portable = if cfg!(target_os = "windows") {
            path.to_string()
        } else {
            path.replace('\\', "/").trim_start_matches("C:").to_string()
        };
        AbsolutePath::new(&portable).unwrap()
    }

    fn test_folder(name: &str, path: &str) -> Folder {
        Folder::new(name, test_path(path)).unwrap()
    }

    fn write_config(
        dir: &tempfile::TempDir,
        entries: Vec<(&str, &str, &str)>,
    ) -> std::path::PathBuf {
        let path = dir.path().join("folders.json");
        let folders: Vec<FolderData> = entries
            .iter()
            .map(|(id, name, p)| FolderData {
                id: id.to_string(),
                name: name.to_string(),
                path: p.to_string(),
                favorite: false,
                order: 0,
                color: None,
                stats: None,
            })
            .collect();
        let config = ConfigFile {
            version: 1,
            folders,
        };
        std::fs::write(&path, serde_json::to_string(&config).unwrap()).unwrap();
        path
    }

    #[tokio::test]
    async fn test_add_skips_exact_duplicate_path() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("folders.json");
        let repo = JsonConfigurationRepository::new(path.clone());

        repo.add(test_folder("Docs", "C:\\Docs")).await.unwrap();
        let dup = test_folder("Docs again", "C:\\Docs");
        assert!(repo.add(dup).await.is_err());

        let folders = repo.load_all().await.unwrap();
        assert_eq!(folders.len(), 1);
    }

    #[tokio::test]
    async fn test_add_skips_duplicate_with_trailing_separator() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("folders.json");
        let repo = JsonConfigurationRepository::new(path.clone());

        repo.add(test_folder("Docs", "C:\\Docs")).await.unwrap();
        let dup = test_folder("Docs slash", "C:\\Docs\\");
        assert!(repo.add(dup).await.is_err());

        let folders = repo.load_all().await.unwrap();
        assert_eq!(folders.len(), 1);
    }

    #[cfg(target_os = "windows")]
    #[tokio::test]
    async fn test_add_skips_duplicate_case_insensitive_on_windows() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("folders.json");
        let repo = JsonConfigurationRepository::new(path.clone());

        repo.add(test_folder("Docs", "C:\\Docs")).await.unwrap();
        let dup = test_folder("Docs lowercase", "c:\\docs");
        assert!(repo.add(dup).await.is_err());

        let folders = repo.load_all().await.unwrap();
        assert_eq!(folders.len(), 1);
    }

    #[tokio::test]
    async fn test_add_allows_distinct_paths() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("folders.json");
        let repo = JsonConfigurationRepository::new(path.clone());

        repo.add(test_folder("Docs", "C:\\Docs")).await.unwrap();
        repo.add(test_folder("Music", "C:\\Music")).await.unwrap();

        let folders = repo.load_all().await.unwrap();
        assert_eq!(folders.len(), 2);
    }

    #[tokio::test]
    async fn test_load_dedup_existing_config() {
        let dir = tempdir().unwrap();
        let path = write_config(
            &dir,
            vec![
                ("11111111-1111-1111-1111-111111111111", "Docs", "C:\\Docs"),
                (
                    "22222222-2222-2222-2222-222222222222",
                    "Docs dup",
                    "C:\\Docs",
                ),
            ],
        );
        let repo = JsonConfigurationRepository::new(path);

        let folders = repo.load_all().await.unwrap();
        assert_eq!(folders.len(), 1);
        assert_eq!(
            folders[0].id.to_string(),
            "11111111-1111-1111-1111-111111111111"
        );
    }

    #[tokio::test]
    async fn test_remove_persists_deletion() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("folders.json");
        let repo = JsonConfigurationRepository::new(path.clone());

        let first = test_folder("Docs", "C:\\Docs");
        repo.add(first.clone()).await.unwrap();
        repo.add(test_folder("Music", "C:\\Music")).await.unwrap();

        repo.remove(&first.id).await.unwrap();

        let folders = repo.load_all().await.unwrap();
        assert_eq!(folders.len(), 1);
        assert_eq!(folders[0].name, "Music");
    }
}
