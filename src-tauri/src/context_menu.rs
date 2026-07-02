use anyhow::{Context, Result};
use winreg::enums::*;
use winreg::RegKey;

pub struct ContextMenuBuilder {
    exe_path: String,
    folders: Vec<crate::config::TargetFolder>,
}

impl ContextMenuBuilder {
    pub fn new(exe_path: &str, folders: &[crate::config::TargetFolder]) -> Self {
        Self {
            exe_path: exe_path.to_string(),
            folders: folders.to_vec(),
        }
    }

    pub fn build(&self) -> Result<()> {
        // Регистрируем меню для файлов (*) и папок (Folder)
        self.register_for_class(r"Software\Classes\*\shell")?;
        self.register_for_class(r"Software\Classes\Folder\shell")?;
        Ok(())
    }

    fn register_for_class(&self, base_path: &str) -> Result<()> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        // 1. Полностью удаляем старую ветку QuickSort (если была)
        if let Ok(parent) = hkcu.open_subkey(base_path) {
            parent.delete_subkey_all("QuickSort").ok();
        }

        if self.folders.is_empty() {
            return Ok(());
        }

        // 2. Создаём корневой ключ QuickSort
        let (quicksort_key, _) = hkcu
            .create_subkey(format!("{}\\QuickSort", base_path))
            .context("Не удалось создать корневой ключ")?;
        quicksort_key.set_value("", &"Quicksort ->")?;
        quicksort_key.set_value("SubCommands", &"")?;
        quicksort_key.set_value("Icon", &format!("{},0", self.exe_path))?;

        // 3. Создаём контейнер shell внутри QuickSort
        let (menu_shell, _) = quicksort_key
            .create_subkey("shell")
            .context("Не удалось создать shell")?;

        // 4. Для каждой папки создаём подпункт с командой ПРЯМО ЗДЕСЬ
        for folder in &self.folders {
            let cmd_name = format!("QuickSort.Move.{}", folder.id);

            // Создаём ключ подпункта
            let (item_key, _) = menu_shell
                .create_subkey(&cmd_name)
                .context("Не удалось создать подпункт")?;
            item_key.set_value("", &folder.name)?; // Имя, отображаемое в меню
            // Можно добавить иконку (опционально)
            // item_key.set_value("Icon", &format!("{},0", self.exe_path))?;

            // Внутри подпункта создаём command
            let (cmd_key, _) = item_key
                .create_subkey("command")
                .context("Не удалось создать command")?;
            let run_cmd = format!(
                "\"{}\" --move --target \"{}\" --file \"%1\"",
                self.exe_path, folder.path
            );
            cmd_key.set_value("", &run_cmd)?;
        }

        Ok(())
    }
}

/// Проверить, существует ли меню (хотя бы в одной ветке)
pub fn is_menu_registered() -> bool {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    hkcu.open_subkey(r"Software\Classes\*\shell\QuickSort").is_ok()
        || hkcu.open_subkey(r"Software\Classes\Folder\shell\QuickSort").is_ok()
}