use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub struct MoveEngine;

impl MoveEngine {
    pub fn move_file(src: &Path, dest_dir: &Path) -> Result<PathBuf> {
        let file_name = src.file_name().context("Не удалось получить имя файла")?;
        let dest = dest_dir.join(file_name);

        match std::fs::rename(src, &dest) {
            Ok(()) => Ok(dest),
            Err(e) if e.raw_os_error() == Some(17) => { // EXDEV
                std::fs::copy(src, &dest)?;
                std::fs::remove_file(src)?;
                Ok(dest)
            }
            Err(e) => Err(e.into()),
        }
    }
}