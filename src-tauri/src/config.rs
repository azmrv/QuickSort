use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TargetFolder {
    pub id: String,
    pub name: String,
    pub path: String,
}

/// Путь к %APPDATA%\QuickSort\folders.json
pub fn get_config_path() -> PathBuf {
    let appdata = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
    let mut path = PathBuf::from(appdata);
    path.push("QuickSort");
    fs::create_dir_all(&path).ok();            // создаём папку, если её нет
    path.push("folders.json");
    path
}

/// Загрузить список папок из JSON. Если файла нет — вернуть пустой вектор.
pub fn load_folders() -> Vec<TargetFolder> {
    let path = get_config_path();
    if path.exists() {
        let data = fs::read_to_string(&path).unwrap_or_default();
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        vec![]
    }
}

/// Сохранить список папок в JSON (с красивым форматированием)
pub fn save_folders(folders: &[TargetFolder]) -> Result<(), String> {
    let path = get_config_path();
    let json = serde_json::to_string_pretty(folders)
        .map_err(|e| format!("Ошибка сериализации: {}", e))?;
    fs::write(&path, json)
        .map_err(|e| format!("Не удалось записать файл: {}", e))?;
    Ok(())
}