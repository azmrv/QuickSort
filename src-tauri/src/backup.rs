//! Backup and restore of user configuration (settings.json + folders.json)
//! as ZIP archives (0.2.6 feature #20 / plan Q7).

use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use zip::write::SimpleFileOptions;
use zip::ZipWriter;

/// File name of the settings entry inside a backup archive.
const ENTRY_SETTINGS: &str = "settings.json";
/// File name of the folders entry inside a backup archive.
const ENTRY_FOLDERS: &str = "folders.json";
/// Maximum number of automatic backups kept before the oldest ones are pruned.
const MAX_AUTO_BACKUPS: usize = 10;

/// Write a ZIP archive with the given entries (archive name → source file).
fn write_archive(target: &Path, entries: &[(&str, &Path)]) -> Result<(), String> {
    let file = File::create(target)
        .map_err(|e| format!("Failed to create backup file {}: {e}", target.display()))?;
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o644);

    for (name, source) in entries {
        let mut content = Vec::new();
        File::open(source)
            .and_then(|mut f| f.read_to_end(&mut content))
            .map_err(|e| format!("Failed to read {}: {e}", source.display()))?;
        zip.start_file(*name, options)
            .map_err(|e| format!("Failed to add {} to archive: {e}", source.display()))?;
        zip.write_all(&content)
            .map_err(|e| format!("Failed to write {} to archive: {e}", source.display()))?;
    }

    zip.finish()
        .map_err(|e| format!("Failed to finalize archive {}: {e}", target.display()))?;
    Ok(())
}

/// Extract the known config entries from a ZIP archive into `out_dir`.
fn read_archive(archive: &Path, out_dir: &Path) -> Result<usize, String> {
    let file = File::open(archive)
        .map_err(|e| format!("Failed to open backup {}: {e}", archive.display()))?;
    let mut zip = zip::ZipArchive::new(file)
        .map_err(|e| format!("Invalid backup archive {}: {e}", archive.display()))?;

    let mut restored = 0usize;
    for i in 0..zip.len() {
        let mut entry = zip
            .by_index(i)
            .map_err(|e| format!("Failed to read archive entry: {e}"))?;
        let name = entry.name().to_string();
        if name != ENTRY_SETTINGS && name != ENTRY_FOLDERS {
            continue;
        }
        let mut content = Vec::new();
        entry
            .read_to_end(&mut content)
            .map_err(|e| format!("Failed to read {name} from archive: {e}"))?;
        let out_path = out_dir.join(&name);
        std::fs::write(&out_path, content)
            .map_err(|e| format!("Failed to write {}: {e}", out_path.display()))?;
        restored += 1;
    }

    if restored == 0 {
        return Err(format!(
            "Backup {} contains neither settings.json nor folders.json",
            archive.display()
        ));
    }
    Ok(restored)
}

/// Create a backup of the current settings.json and folders.json.
///
/// Only files that exist on disk are included; at least one must exist.
/// Returns the canonical path of the created archive.
pub fn create_backup(target_path: &Path) -> Result<PathBuf, String> {
    let settings = crate::platform::paths::settings_config_path();
    let folders = crate::platform::paths::folders_config_path();
    let entries: Vec<(&str, &Path)> = [
        (ENTRY_SETTINGS, settings.as_path()),
        (ENTRY_FOLDERS, folders.as_path()),
    ]
    .into_iter()
    .filter(|(_, p)| p.exists())
    .collect();
    if entries.is_empty() {
        return Err("No configuration files found to back up".to_string());
    }
    write_archive(target_path, &entries)?;
    Ok(target_path.to_path_buf())
}

/// Restore settings.json and folders.json from a backup archive.
///
/// Returns the number of restored files.
pub fn restore_backup(backup_path: &Path) -> Result<usize, String> {
    let out_dir = crate::platform::paths::config_dir();
    read_archive(backup_path, &out_dir)
}

/// Create an automatic backup in the data directory and prune old archives.
///
/// Auto-backups use a timestamped name so the last `MAX_AUTO_BACKUPS` archives
/// survive a session.
pub fn auto_backup() -> Result<PathBuf, String> {
    let dir = crate::platform::paths::data_dir().join("backups");
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("Failed to create backup directory {}: {e}", dir.display()))?;
    let stamp = chrono::Utc::now().format("%Y%m%d-%H%M%S");
    let target = dir.join(format!("auto-{stamp}.zip"));
    create_backup(&target)?;
    prune_old(&dir, MAX_AUTO_BACKUPS);
    Ok(target)
}

/// Delete the oldest ZIP archives in `dir` beyond `keep` (lexicographic order
/// matches the timestamped names used by `auto_backup`).
fn prune_old(dir: &Path, keep: usize) {
    let mut archives: Vec<PathBuf> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "zip"))
        .collect();
    archives.sort();
    for old in archives.iter().take(archives.len().saturating_sub(keep)) {
        let _ = std::fs::remove_file(old);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backup_round_trip_preserves_config_files() {
        let dir = std::env::temp_dir().join(format!("qs-backup-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let settings = dir.join("settings.json");
        let folders = dir.join("folders.json");
        std::fs::write(&settings, br#"{"theme_mode":"dark"}"#).unwrap();
        std::fs::write(&folders, br#"[]"#).unwrap();

        let archive = dir.join("backup.zip");
        write_archive(
            &archive,
            &[
                (ENTRY_SETTINGS, settings.as_path()),
                (ENTRY_FOLDERS, folders.as_path()),
            ],
        )
        .unwrap();

        let out = dir.join("out");
        std::fs::create_dir_all(&out).unwrap();
        let restored = read_archive(&archive, &out).unwrap();
        assert_eq!(restored, 2);
        assert_eq!(
            std::fs::read(out.join("settings.json")).unwrap(),
            br#"{"theme_mode":"dark"}"#
        );
        assert_eq!(std::fs::read(out.join("folders.json")).unwrap(), br#"[]"#);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn missing_config_files_are_skipped() {
        let dir = std::env::temp_dir().join(format!("qs-backup-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let settings = dir.join("settings.json");
        std::fs::write(&settings, br#"{}"#).unwrap();
        let missing = dir.join("folders.json");

        let archive = dir.join("backup.zip");
        write_archive(&archive, &[(ENTRY_FOLDERS, missing.as_path())]).unwrap_err();

        let entries = [
            (ENTRY_SETTINGS, settings.as_path()),
            (ENTRY_FOLDERS, missing.as_path()),
        ]
        .into_iter()
        .filter(|(_, p)| p.exists())
        .collect::<Vec<_>>();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0, ENTRY_SETTINGS);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn restore_rejects_archive_without_config_entries() {
        let dir = std::env::temp_dir().join(format!("qs-backup-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let random = dir.join("random.txt");
        std::fs::write(&random, b"hello").unwrap();
        let archive = dir.join("backup.zip");
        write_archive(&archive, &[("random.txt", random.as_path())]).unwrap();

        let err = read_archive(&archive, &dir).unwrap_err();
        assert!(err.contains("neither settings.json nor folders.json"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn prune_old_keeps_only_latest() {
        let dir = std::env::temp_dir().join(format!("qs-backup-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        for name in [
            "auto-20260101-000000.zip",
            "auto-20260102-000000.zip",
            "auto-20260103-000000.zip",
        ] {
            std::fs::write(dir.join(name), b"x").unwrap();
        }
        prune_old(&dir, 2);
        let remaining: Vec<String> = std::fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|x| x == "zip"))
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect();
        assert_eq!(remaining.len(), 2);
        assert!(remaining.contains(&"auto-20260102-000000.zip".to_string()));
        assert!(remaining.contains(&"auto-20260103-000000.zip".to_string()));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
