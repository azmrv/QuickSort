use anyhow::Result;
use crate::models::{Config, Folder, FolderId};
use super::repository::FolderRepository;

pub struct FolderService<R: FolderRepository> {
    repo: R,
}

impl<R: FolderRepository> FolderService<R> {
    pub fn new(repo: R) -> Self { Self { repo } }

    pub fn list(&self) -> Result<Vec<Folder>> {
        let config = self.repo.load()?;
        Ok(config.folders)
    }

    pub fn update_all(&self, folders: Vec<Folder>) -> Result<()> {
        let config = Config { version: 1, folders };
        self.repo.save(&config)
    }

    pub fn toggle_favorite(&self, id: FolderId) -> Result<()> {
        let mut config = self.repo.load()?;
        if let Some(f) = config.folders.iter_mut().find(|f| f.id == id) {
            f.favorite = !f.favorite;
        }
        self.repo.save(&config)
    }
}