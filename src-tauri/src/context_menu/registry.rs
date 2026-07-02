use anyhow::Result;
use win_ctx::{CtxEntry, ActivationType, EntryOptions};

pub struct RegistryInstaller;

impl RegistryInstaller {
    pub fn install(model: &super::model::MenuModel, exe_path: &str) -> Result<()> {
        // Удаляем старые ключи (если были)
        Self::uninstall()?;

        // Создаём корневое меню для всех файлов и папок
        let root_files = CtxEntry::new("QuickSort", &ActivationType::File("*".into()))?;
        let root_folders = CtxEntry::new("QuickSort", &ActivationType::Folder)?;

        for item in &model.items {
            match item {
                super::model::MenuItem::Favorite { name, target, .. } => {
                    let cmd = format!("\"{}\" move \"{}\" \"%1\"", exe_path, target);
                    let opts = EntryOptions {
                        command: Some(cmd),
                        icon: Some(format!("{},0", exe_path)),
                        position: None,
                        separator: None,
                        extended: false,
                    };
                    root_files.new_child_with_options(name, &opts)?;
                    root_folders.new_child_with_options(name, &opts)?;
                }
                super::model::MenuItem::More => {
                    let cmd = format!("\"{}\" select-folder \"%1\"", exe_path);
                    let opts = EntryOptions {
                        command: Some(cmd),
                        icon: Some(format!("{},0", exe_path)),
                        position: None,
                        separator: None,
                        extended: false,
                    };
                    root_files.new_child_with_options("📂 Другие папки...", &opts)?;
                    root_folders.new_child_with_options("📂 Другие папки...", &opts)?;
                }
            }
        }

        Ok(())
    }

    pub fn uninstall() -> Result<()> {
        // Удаляем корневые записи, если они существуют (рекурсивно удалят и детей)
        if let Some(entry) = CtxEntry::get(&["QuickSort"], &ActivationType::File("*".into())) {
            entry.delete()?;
        }
        if let Some(entry) = CtxEntry::get(&["QuickSort"], &ActivationType::Folder) {
            entry.delete()?;
        }
        Ok(())
    }
}