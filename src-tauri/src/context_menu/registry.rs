use anyhow::Result;
use winreg::enums::*;
use winreg::RegKey;
use super::model::{MenuModel, MenuItem};

pub struct RegistryInstaller;

impl RegistryInstaller {
    pub fn install(model: &MenuModel, exe_path: &str) -> Result<()> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        Self::install_to(&hkcu, r"Software\Classes\*\shell\QuickSort", model, exe_path)?;
        Self::install_to(&hkcu, r"Software\Classes\Folder\shell\QuickSort", model, exe_path)?;
        Ok(())
    }

    fn install_to(hkcu: &RegKey, base: &str, model: &MenuModel, exe_path: &str) -> Result<()> {
        // Удаляем старый ключ
        if let Ok(parent) = hkcu.open_subkey(base) {
            parent.delete_subkey_all("").ok();
        }
        let (key, _) = hkcu.create_subkey(base)?;
        let (shell, _) = key.create_subkey("shell")?;

        for item in &model.items {
            match item {
                MenuItem::Favorite { id, name, target } => {
                    let (cmd_key, _) = shell.create_subkey(id)?;
                    cmd_key.set_value("", &name.as_str())?;
                    let (command, _) = cmd_key.create_subkey("command")?;
                    command.set_value("", &format!("\"{}\" --move --target \"{}\" --file \"%1\"", exe_path, target))?;
                }
                MenuItem::More => {
                    let (cmd_key, _) = shell.create_subkey("zzz_more")?;
                    cmd_key.set_value("", &"📂 Другие папки...")?;
                    let (command, _) = cmd_key.create_subkey("command")?;
                    command.set_value("", &format!("\"{}\" --select-folder --file \"%1\"", exe_path))?;
                }
            }
        }
        Ok(())
    }

    pub fn uninstall() -> Result<()> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        for base in &[r"Software\Classes\*\shell\QuickSort", r"Software\Classes\Folder\shell\QuickSort"] {
            if let Ok(parent) = hkcu.open_subkey(base) {
                parent.delete_subkey_all("").ok();
            }
        }
        Ok(())
    }
}