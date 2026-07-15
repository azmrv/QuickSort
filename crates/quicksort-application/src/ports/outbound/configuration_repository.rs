//! Outbound port for configuration storage (folders, settings).

use async_trait::async_trait;
use quicksort_domain::{Folder, FolderId};
use crate::errors::UseCaseError;

#[async_trait]
pub trait ConfigurationRepository: Send + Sync {
    /// Загружает все сохранённые папки
    async fn load_all(&self) -> Result<Vec<Folder>, UseCaseError>;
    /// Сохраняет все папки
    async fn save_all(&self, folders: &[Folder]) -> Result<(), UseCaseError>;
    /// Добавляет новую папку
    async fn add(&self, folder: Folder) -> Result<(), UseCaseError>;
    /// Удаляет папку по ID
    async fn remove(&self, id: &FolderId) -> Result<(), UseCaseError>;
    /// Находит папку по ID
    async fn find_by_id(&self, id: &FolderId) -> Result<Option<Folder>, UseCaseError>;
    /// Находит папку по полному пути
    async fn find_by_path(&self, path: &str) -> Result<Option<Folder>, UseCaseError>;
    /// Возвращает ID папки "Документы" (по умолчанию)
    async fn get_default_folder_id(&self) -> Result<FolderId, UseCaseError>;
}
