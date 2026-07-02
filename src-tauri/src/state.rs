use parking_lot::Mutex;
use crate::models::Config;
use crate::folder::repository::JsonRepository;
use crate::folder::service::FolderService;

pub struct AppState {
    pub service: FolderService<JsonRepository>,
    pub config: Mutex<Config>, // кэш текущего конфига
}