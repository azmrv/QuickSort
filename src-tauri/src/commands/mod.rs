use crate::state::{AppState, SystemInfoSample};
use quicksort_application::{
    AbsolutePath, ExecuteOperation, Folder, FolderId, FolderMetadata, GetFolders,
    GetOperationHistory, LoadSettings, LogLevel, ManageFolders, OperationErrorDto, OperationId,
    PluginConfig, PluginInfoDto, PluginManager, SaveSettings, Settings, UndoErrorKind,
    UndoOperation,
};
use serde::Serialize;
use std::path::PathBuf;
use sysinfo::{Components, DiskRefreshKind, Disks, Networks, System};
use tauri::{AppHandle, State};

/// A folder paired with live file-system metadata for the UI.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct FolderWithMetadata {
    pub folder: Folder,
    pub metadata: FolderMetadata,
}

/// Outcome of adding folders by dropping them onto the Folder tab.
/// `skipped` holds per-path human-readable reasons (invalid path,
/// not a directory, duplicate, invalid name).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct AddFoldersFromPathsResult {
    pub added: usize,
    pub skipped: Vec<String>,
}

#[tauri::command]
pub async fn get_folders_v2(state: State<'_, AppState>) -> Result<Vec<Folder>, String> {
    tracing::info!(command = "get_folders_v2", "handling");
    let result = state.facade.get_all().await.map_err(|e| e.to_string());
    match &result {
        Ok(folders) => tracing::info!(command = "get_folders_v2", count = folders.len(), "OK"),
        Err(e) => tracing::error!(command = "get_folders_v2", error = %e, "FAIL"),
    }
    result
}

#[tauri::command]
pub async fn get_folders_with_metadata(
    state: State<'_, AppState>,
) -> Result<Vec<FolderWithMetadata>, String> {
    tracing::info!(command = "get_folders_with_metadata", "handling");
    let folders = state.facade.get_all().await.map_err(|e| e.to_string())?;

    let mut result = Vec::with_capacity(folders.len());
    for folder in folders {
        let metadata = state
            .fs
            .folder_metadata(&folder.path)
            .await
            .map_err(|e| e.to_string())?;
        result.push(FolderWithMetadata { folder, metadata });
    }

    tracing::info!(
        command = "get_folders_with_metadata",
        count = result.len(),
        "OK"
    );
    Ok(result)
}

#[tauri::command]
pub async fn add_folder_v2(
    state: State<'_, AppState>,
    name: String,
    path: String,
) -> Result<(), String> {
    tracing::info!(command = "add_folder_v2", name = %name, path = %path, "handling");
    let windows_path = AbsolutePath::new(&path).map_err(|e| {
        tracing::error!(command = "add_folder_v2", error = %e, "invalid path");
        format!("Invalid path: {}", e)
    })?;
    let folder = Folder::new(&name, windows_path).map_err(|e| {
        tracing::error!(command = "add_folder_v2", error = %e, "invalid folder");
        format!("Invalid folder: {}", e)
    })?;
    let result = state
        .facade
        .add_folder(folder)
        .await
        .map_err(|e| e.to_string());
    match &result {
        Ok(()) => {
            tracing::info!(command = "add_folder_v2", "OK");
            auto_backup_after_change();
        }
        Err(e) => tracing::error!(command = "add_folder_v2", error = %e, "FAIL"),
    }
    result
}

/// Add several folders by absolute path in one call.
///
/// Used by the Folder tab drag-and-drop: the frontend receives the raw
/// dropped paths from Explorer and sends them here verbatim. Only existing
/// directories are added; every other path (file, invalid path, duplicate,
/// invalid name) is reported in `skipped` instead of failing the whole batch.
#[tauri::command]
pub async fn add_folders_from_paths(
    state: State<'_, AppState>,
    paths: Vec<String>,
) -> Result<AddFoldersFromPathsResult, String> {
    tracing::info!(
        command = "add_folders_from_paths",
        count = paths.len(),
        "handling"
    );

    let mut added = 0usize;
    let mut skipped: Vec<String> = Vec::new();

    for raw_path in paths {
        let windows_path = match AbsolutePath::new(&raw_path) {
            Ok(path) => path,
            Err(e) => {
                skipped.push(format!("{}: {}", raw_path, e));
                continue;
            }
        };

        match state.fs.is_dir(&windows_path).await {
            Ok(true) => {}
            Ok(false) => {
                skipped.push(format!("{}: not a directory", raw_path));
                continue;
            }
            Err(e) => {
                skipped.push(format!("{}: {}", raw_path, e));
                continue;
            }
        }

        let name = match windows_path.file_name() {
            Some(name) => name.to_string(),
            None => {
                skipped.push(format!("{}: no folder name", raw_path));
                continue;
            }
        };

        let folder = match Folder::new(&name, windows_path) {
            Ok(folder) => folder,
            Err(e) => {
                skipped.push(format!("{}: {}", raw_path, e));
                continue;
            }
        };

        match state.facade.add_folder(folder).await {
            Ok(()) => added += 1,
            Err(e) => {
                skipped.push(format!("{}: {}", raw_path, e));
            }
        }
    }

    if added > 0 {
        auto_backup_after_change();
    }
    tracing::info!(
        command = "add_folders_from_paths",
        added = added,
        skipped = skipped.len(),
        "OK"
    );
    Ok(AddFoldersFromPathsResult { added, skipped })
}

#[tauri::command]
pub async fn remove_folder_v2(state: State<'_, AppState>, id: String) -> Result<(), String> {
    tracing::info!(command = "remove_folder_v2", id = %id, "handling");
    let folder_id = FolderId::from_string(&id).map_err(|e| {
        tracing::error!(command = "remove_folder_v2", error = %e, "invalid folder ID");
        format!("Invalid folder ID: {}", e)
    })?;
    let result = state
        .facade
        .remove_folder(folder_id)
        .await
        .map_err(|e| e.to_string());
    match &result {
        Ok(()) => {
            tracing::info!(command = "remove_folder_v2", "OK");
            auto_backup_after_change();
        }
        Err(e) => tracing::error!(command = "remove_folder_v2", error = %e, "FAIL"),
    }
    result
}

#[tauri::command]
pub async fn toggle_favorite_v2(
    state: State<'_, AppState>,
    id: String,
    order: Option<u32>,
) -> Result<(), String> {
    tracing::info!(command = "toggle_favorite_v2", id = %id, order = ?order, "handling");
    let _ = order; // TODO: support order reordering in toggle_favorite port
    let folder_id = FolderId::from_string(&id).map_err(|e| {
        tracing::error!(command = "toggle_favorite_v2", error = %e, "invalid folder ID");
        format!("Invalid folder ID: {}", e)
    })?;
    let result = state
        .facade
        .toggle_favorite(folder_id)
        .await
        .map_err(|e| e.to_string());
    match &result {
        Ok(()) => {
            tracing::info!(command = "toggle_favorite_v2", "OK");
            auto_backup_after_change();
        }
        Err(e) => tracing::error!(command = "toggle_favorite_v2", error = %e, "FAIL"),
    }
    result
}

#[tauri::command]
pub async fn set_folder_color_v2(
    state: State<'_, AppState>,
    id: String,
    color: Option<String>,
) -> Result<(), String> {
    tracing::info!(command = "set_folder_color_v2", id = %id, color = ?color, "handling");
    let folder_id = FolderId::from_string(&id).map_err(|e| {
        tracing::error!(command = "set_folder_color_v2", error = %e, "invalid folder ID");
        format!("Invalid folder ID: {}", e)
    })?;
    let result = state
        .facade
        .set_folder_color(folder_id, color)
        .await
        .map_err(|e| e.to_string());
    match &result {
        Ok(()) => {
            tracing::info!(command = "set_folder_color_v2", "OK");
            auto_backup_after_change();
        }
        Err(e) => tracing::error!(command = "set_folder_color_v2", error = %e, "FAIL"),
    }
    result
}

#[tauri::command]
pub async fn execute_operation_v2(
    state: State<'_, AppState>,
    command: quicksort_application::OperationCommand,
) -> Result<quicksort_application::OperationResult, String> {
    tracing::info!(command = "execute_operation_v2", op_type = ?command.operation_type, sources = ?command.source_paths, "handling");
    let result = state
        .facade
        .execute(command)
        .await
        .map_err(|e| e.to_string());
    match &result {
        Ok(r) => {
            tracing::info!(command = "execute_operation_v2", state = ?r.state, files = r.processed_files, "OK")
        }
        Err(e) => tracing::error!(command = "execute_operation_v2", error = %e, "FAIL"),
    }
    result
}

/// Records a user intent (file selection) for audit logging.
///
/// Called by the frontend Search/Selector pages when the user picks files
/// or a target folder. Generates a `correlation_id` when the caller does
/// not provide one and returns it, so the caller can attach the same id to
/// the subsequently enqueued operation (spec #15, release 0.2.6).
#[tauri::command]
pub fn log_user_select(
    source: String,
    paths: Vec<String>,
    correlation_id: Option<String>,
) -> Result<String, String> {
    let correlation_id = correlation_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let src = correlation_id.clone();
    let source_map = match source.as_str() {
        "Search" => "search",
        "Selector" => "selector",
        "ContextMenu" => "context_menu",
        _ => "api",
    };
    tracing::info!(
        event = "user-select",
        source = source_map,
        file_count = paths.len(),
        correlation_id = %src,
        paths = ?paths,
        "user intent recorded"
    );
    Ok(correlation_id)
}

#[tauri::command]
pub async fn undo_operation_v2(
    state: State<'_, AppState>,
    operation_id: String,
) -> Result<quicksort_application::OperationResult, OperationErrorDto> {
    tracing::info!(command = "undo_operation_v2", operation_id = %operation_id, "handling");
    let id = OperationId::from_string(&operation_id).map_err(|e| {
        tracing::error!(command = "undo_operation_v2", error = %e, "invalid operation ID");
        OperationErrorDto {
            kind: UndoErrorKind::Unknown,
            message: format!("Invalid operation ID: {e}"),
        }
    })?;
    let result = state
        .facade
        .undo(id)
        .await
        .map_err(|e| e.to_operation_error());
    match &result {
        Ok(r) => tracing::info!(command = "undo_operation_v2", state = ?r.state, "OK"),
        Err(e) => tracing::error!(
            command = "undo_operation_v2",
            kind = ?e.kind,
            message = %e.message,
            "FAIL"
        ),
    }
    result
}

/// Re-executes a previously completed operation with the same parameters.
///
/// The operation history only stores the target folder *path*, so the
/// current folder configuration is searched for a folder with a matching
/// path to resolve the `FolderId` required by `OperationCommand`.
#[tauri::command]
pub async fn repeat_operation_v2(
    state: State<'_, AppState>,
    operation_id: String,
) -> Result<quicksort_application::OperationResult, OperationErrorDto> {
    tracing::info!(command = "repeat_operation_v2", operation_id = %operation_id, "handling");
    let id = OperationId::from_string(&operation_id).map_err(|e| {
        tracing::error!(command = "repeat_operation_v2", error = %e, "invalid operation ID");
        OperationErrorDto {
            kind: UndoErrorKind::Unknown,
            message: format!("Invalid operation ID: {e}"),
        }
    })?;

    // Find the original operation in the in-memory history.
    let operations = state
        .facade
        .get_all_operations()
        .await
        .map_err(|e| e.to_operation_error())?;
    let original = match operations.iter().find(|op| op.id == id) {
        Some(op) => op.clone(),
        None => {
            tracing::error!(command = "repeat_operation_v2", "operation not found");
            return Err(OperationErrorDto {
                kind: UndoErrorKind::Unknown,
                message: format!("Operation not found: {}", operation_id),
            });
        }
    };

    // Resolve the target folder ID by matching the stored target path.
    // Not required for Delete/Rename, which carry no target folder.
    let target_folder_id = match &original.target_folder_path {
        Some(target_path) => {
            let folders = state
                .facade
                .get_all()
                .await
                .map_err(|e| e.to_operation_error())?;
            match folders.iter().find(|f| &f.path == target_path) {
                Some(folder) => Some(folder.id),
                None => {
                    tracing::error!(
                        command = "repeat_operation_v2",
                        path = %target_path,
                        "target folder not found"
                    );
                    return Err(OperationErrorDto {
                        kind: UndoErrorKind::Unknown,
                        message: format!("Target folder not found: {}", target_path),
                    });
                }
            }
        }
        None => None,
    };

    // Rebuild the original command and execute it again. The source is
    // inherited so the Operations UI keeps labelling repeats correctly;
    // a fresh correlation_id is generated for the new intent (spec #15).
    let command = quicksort_application::OperationCommand {
        operation_type: original.operation_type,
        source_paths: original.source_paths,
        target_folder_id,
        target_paths: original.target_paths,
        overwrite_policy: quicksort_application::OverwritePolicy::Skip,
        duplicate_check_mode: quicksort_application::DuplicateCheckMode::default(),
        source: original.source,
        correlation_id: uuid::Uuid::new_v4(),
    };

    let result = state
        .facade
        .execute(command)
        .await
        .map_err(|e| e.to_operation_error());
    match &result {
        Ok(r) => tracing::info!(
            command = "repeat_operation_v2",
            state = ?r.state,
            files = r.processed_files,
            "OK"
        ),
        Err(e) => tracing::error!(
            command = "repeat_operation_v2",
            kind = ?e.kind,
            message = %e.message,
            "FAIL"
        ),
    }
    result
}

#[tauri::command]
pub fn get_mode() -> String {
    tracing::debug!(command = "get_mode", "handling");
    "Editor".to_string()
}

#[tauri::command]
pub fn get_pending_file() -> Option<String> {
    tracing::info!(command = "get_pending_file", "handling");
    let file = crate::pending::get_pending_file();
    tracing::info!(command = "get_pending_file", file = ?file, "OK");
    file
}

#[tauri::command]
pub fn get_pending_files() -> Vec<String> {
    tracing::info!(command = "get_pending_files", "handling");
    let files = crate::pending::get_pending_files();
    tracing::info!(command = "get_pending_files", count = files.len(), "OK");
    files
}

#[tauri::command]
pub fn check_menu_status() -> bool {
    tracing::debug!(command = "check_menu_status", "handling");
    #[cfg(target_os = "windows")]
    {
        crate::com::is_registered()
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

#[tauri::command]
pub fn get_logs() -> Vec<serde_json::Value> {
    crate::logging::get_recent_logs()
}

#[tauri::command]
pub fn set_log_level(level: LogLevel) -> Result<(), String> {
    tracing::info!(command = "set_log_level", level = ?level, "handling");
    let result = crate::logging::set_log_level(level);
    match &result {
        Ok(()) => tracing::info!(command = "set_log_level", "OK"),
        Err(e) => tracing::error!(command = "set_log_level", error = %e, "FAIL"),
    }
    result
}

#[tauri::command]
pub fn register_com_server() -> Result<String, String> {
    tracing::info!(command = "register_com_server", "handling");
    #[cfg(target_os = "windows")]
    {
        let was_active = matches!(
            crate::com::check_registration(),
            crate::com::RegistrationStatus::PathMismatch { .. }
        );
        crate::com::register(was_active)?;
        tracing::info!(
            command = "register_com_server",
            "OK — registry keys written"
        );
        Ok("COM server registered successfully.".to_string())
    }
    #[cfg(not(target_os = "windows"))]
    {
        Err("COM registration is only supported on Windows".to_string())
    }
}

#[tauri::command]
pub fn unregister_com_server() -> Result<String, String> {
    tracing::info!(command = "unregister_com_server", "handling");
    #[cfg(target_os = "windows")]
    {
        crate::com::unregister(true)?;
        tracing::info!(command = "unregister_com_server", "OK");
        Ok("COM server unregistered successfully.".to_string())
    }
    #[cfg(not(target_os = "windows"))]
    {
        Err("COM unregistration is only supported on Windows".to_string())
    }
}

#[tauri::command]
pub fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<Settings, String> {
    tracing::info!(command = "get_settings", "handling");
    let result = state
        .facade
        .load_settings()
        .await
        .map_err(|e| e.to_string());
    match &result {
        Ok(_) => tracing::info!(command = "get_settings", "OK"),
        Err(e) => tracing::error!(command = "get_settings", error = %e, "FAIL"),
    }
    result
}

#[tauri::command]
pub async fn save_settings(state: State<'_, AppState>, settings: Settings) -> Result<(), String> {
    tracing::info!(command = "save_settings", "handling");
    let result = state
        .facade
        .save_settings(settings)
        .await
        .map_err(|e| e.to_string());
    match &result {
        Ok(()) => {
            tracing::info!(command = "save_settings", "OK");
            auto_backup_after_change();
        }
        Err(e) => tracing::error!(command = "save_settings", error = %e, "FAIL"),
    }
    result
}

#[tauri::command]
pub async fn get_operations(
    state: State<'_, AppState>,
) -> Result<Vec<quicksort_application::Operation>, String> {
    tracing::info!(command = "get_operations", "handling");
    let result = state
        .facade
        .get_all_operations()
        .await
        .map_err(|e| e.to_string());
    match &result {
        Ok(ops) => tracing::info!(command = "get_operations", count = ops.len(), "OK"),
        Err(e) => tracing::error!(command = "get_operations", error = %e, "FAIL"),
    }
    result
}

/// Delete a single operation from the history by its identifier.
#[tauri::command]
pub async fn delete_operation(
    state: State<'_, AppState>,
    operation_id: String,
) -> Result<(), String> {
    tracing::info!(command = "delete_operation", operation_id = %operation_id, "handling");
    let id = OperationId::from_string(&operation_id).map_err(|e| {
        tracing::error!(command = "delete_operation", error = %e, "invalid operation ID");
        format!("Invalid operation ID: {}", e)
    })?;
    let result = state
        .facade
        .delete_operation(id)
        .await
        .map_err(|e| e.to_string());
    match &result {
        Ok(()) => tracing::info!(command = "delete_operation", "OK"),
        Err(e) => tracing::error!(command = "delete_operation", error = %e, "FAIL"),
    }
    result
}

/// Clear the entire operation history.
#[tauri::command]
pub async fn clear_history(state: State<'_, AppState>) -> Result<(), String> {
    tracing::info!(command = "clear_history", "handling");
    let result = state
        .facade
        .clear_history()
        .await
        .map_err(|e| e.to_string());
    match &result {
        Ok(()) => tracing::info!(command = "clear_history", "OK"),
        Err(e) => tracing::error!(command = "clear_history", error = %e, "FAIL"),
    }
    result
}

/// Launch TeraCopy with the given file list.
///
/// Writes file paths to a temp file and invokes TeraCopy.
/// Supports TeraCopy 3.17 and 4.0.
#[tauri::command]
pub async fn launch_teracopy(files: Vec<String>) -> Result<(), String> {
    tracing::info!(command = "launch_teracopy", count = files.len(), "handling");

    #[cfg(target_os = "windows")]
    {
        // Default TeraCopy paths
        let teracopy_paths = [
            "C:\\Program Files\\TeraCopy\\TeraCopy.exe",
            "C:\\Program Files (x86)\\TeraCopy\\TeraCopy.exe",
        ];

        let teracopy_exe = teracopy_paths
            .iter()
            .find(|p| PathBuf::from(p).exists())
            .ok_or("TeraCopy not found at standard paths")?;

        // Write file list to temp file (Windows-1251 encoding for TeraCopy compatibility)
        let temp_dir = std::env::temp_dir();
        let list_path = temp_dir.join("quicksort_tc_list.txt");
        let content = files.join("\n");
        std::fs::write(&list_path, &content)
            .map_err(|e| format!("Failed to write temp file: {}", e))?;

        // Launch TeraCopy
        std::process::Command::new(teracopy_exe)
            .arg("AddList")
            .arg(format!("*\"{}\"", list_path.to_string_lossy()))
            .spawn()
            .map_err(|e| format!("Failed to launch TeraCopy: {}", e))?;

        tracing::info!(command = "launch_teracopy", "OK");
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        Err("TeraCopy integration is only supported on Windows".to_string())
    }
}

/// Check if TeraCopy is installed on the system.
#[tauri::command]
pub fn check_teracopy_installed() -> bool {
    #[cfg(target_os = "windows")]
    {
        let teracopy_paths = [
            "C:\\Program Files\\TeraCopy\\TeraCopy.exe",
            "C:\\Program Files (x86)\\TeraCopy\\TeraCopy.exe",
        ];
        teracopy_paths.iter().any(|p| PathBuf::from(p).exists())
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

/// Create a new folder in the specified parent directory.
///
/// Returns the path of the created folder.
#[tauri::command]
pub async fn create_new_folder(parent_path: String, folder_name: String) -> Result<String, String> {
    tracing::info!(
        command = "create_new_folder",
        parent = %parent_path,
        name = %folder_name,
        "handling"
    );

    let parent = PathBuf::from(&parent_path);
    if !parent.exists() {
        return Err(format!("Parent directory does not exist: {}", parent_path));
    }

    let new_folder = parent.join(&folder_name);
    if new_folder.exists() {
        return Err(format!("Folder already exists: {}", new_folder.display()));
    }

    std::fs::create_dir(&new_folder).map_err(|e| format!("Failed to create folder: {}", e))?;

    let path_str = new_folder.to_string_lossy().to_string();
    tracing::info!(command = "create_new_folder", path = %path_str, "OK");
    Ok(path_str)
}

// ---------------------------------------------------------------------------
// Plugin management commands
// ---------------------------------------------------------------------------

/// List all discovered plugins.
#[tauri::command]
pub async fn list_plugins(state: State<'_, AppState>) -> Result<Vec<PluginInfoDto>, String> {
    tracing::info!(command = "list_plugins", "handling");
    let result = state.facade.list_plugins().await.map_err(|e| e.to_string());
    match &result {
        Ok(plugins) => tracing::info!(command = "list_plugins", count = plugins.len(), "OK"),
        Err(e) => tracing::error!(command = "list_plugins", error = %e, "FAIL"),
    }
    result
}

/// Get plugin configuration.
#[tauri::command]
pub async fn get_plugin_config(
    state: State<'_, AppState>,
    plugin_id: String,
) -> Result<PluginConfig, String> {
    tracing::info!(command = "get_plugin_config", plugin_id = %plugin_id, "handling");
    let result = state
        .facade
        .get_plugin_config(&plugin_id)
        .await
        .map_err(|e| e.to_string());
    match &result {
        Ok(_) => tracing::info!(command = "get_plugin_config", "OK"),
        Err(e) => tracing::error!(command = "get_plugin_config", error = %e, "FAIL"),
    }
    result
}

/// Save plugin configuration.
#[tauri::command]
pub async fn save_plugin_config(
    state: State<'_, AppState>,
    plugin_id: String,
    config: PluginConfig,
) -> Result<(), String> {
    tracing::info!(command = "save_plugin_config", plugin_id = %plugin_id, "handling");
    let result = state
        .facade
        .save_plugin_config(&plugin_id, config)
        .await
        .map_err(|e| e.to_string());
    match &result {
        Ok(()) => tracing::info!(command = "save_plugin_config", "OK"),
        Err(e) => tracing::error!(command = "save_plugin_config", error = %e, "FAIL"),
    }
    result
}

/// Enable or disable a plugin.
#[tauri::command]
pub async fn set_plugin_enabled(
    state: State<'_, AppState>,
    plugin_id: String,
    enabled: bool,
) -> Result<(), String> {
    tracing::info!(command = "set_plugin_enabled", plugin_id = %plugin_id, enabled = enabled, "handling");
    let result = state
        .facade
        .set_plugin_enabled(&plugin_id, enabled)
        .await
        .map_err(|e| e.to_string());
    match &result {
        Ok(()) => tracing::info!(command = "set_plugin_enabled", "OK"),
        Err(e) => tracing::error!(command = "set_plugin_enabled", error = %e, "FAIL"),
    }
    result
}

/// Rescan plugin directory.
#[tauri::command]
pub async fn rescan_plugins(state: State<'_, AppState>) -> Result<Vec<PluginInfoDto>, String> {
    tracing::info!(command = "rescan_plugins", "handling");
    let result = state
        .facade
        .rescan_plugins()
        .await
        .map_err(|e| e.to_string());
    match &result {
        Ok(plugins) => tracing::info!(command = "rescan_plugins", count = plugins.len(), "OK"),
        Err(e) => tracing::error!(command = "rescan_plugins", error = %e, "FAIL"),
    }
    result
}

// ---------------------------------------------------------------------------
// Search commands
// ---------------------------------------------------------------------------

/// Search for files matching the given query.
#[tauri::command]
pub async fn search_files(
    state: State<'_, AppState>,
    query: String,
    directories: Vec<String>,
) -> Result<quicksort_application::SearchResult, String> {
    tracing::info!(command = "search_files", query = %query, dir_count = directories.len(), "handling");
    let result = state
        .facade
        .search_files(&query, &directories)
        .await
        .map_err(|e| e.to_string());
    match &result {
        Ok(r) => tracing::info!(
            command = "search_files",
            total = r.total_count,
            time_ms = r.search_time_ms,
            "OK"
        ),
        Err(e) => tracing::error!(command = "search_files", error = %e, "FAIL"),
    }
    result
}

// ---------------------------------------------------------------------------
// Metadata command
// ---------------------------------------------------------------------------

/// Returns the complete application metadata (version, authors, credits, etc.).
#[tauri::command]
pub fn get_app_metadata() -> crate::metadata::AppMetadata {
    tracing::debug!(command = "get_app_metadata", "handling");
    crate::metadata::get_metadata()
}

/// Fully quit the application: cleanup resources and exit.
/// Unlike the close button (which hides to tray), this terminates the process.
#[tauri::command]
pub async fn quit_app(app: AppHandle) -> Result<(), String> {
    tracing::info!("quit_app command — performing full shutdown");
    #[cfg(target_os = "windows")]
    {
        let pid_path = crate::platform::paths::pid_file_path();
        match std::fs::remove_file(&pid_path) {
            Ok(()) => tracing::info!("owner PID file removed"),
            Err(e) => tracing::debug!(error = %e, "PID file already absent"),
        }
    }

    // Best-effort COM cleanup: remove registry keys so Explorer stops loading
    // the DLL. Explorer is NOT restarted here — on every app exit that caused the
    // reported endless Explorer restart loop.
    #[cfg(target_os = "windows")]
    {
        let _ = crate::com::unregister(false);
    }

    tracing::info!("all cleanup done, exiting");
    app.exit(0);
    Ok(())
}

// ---------------------------------------------------------------------------
// Queue commands
// ---------------------------------------------------------------------------

/// Enqueue a file operation for asynchronous execution by the persistent
/// job worker. Returns the generated job id.
#[tauri::command]
pub async fn enqueue_operation(
    state: State<'_, AppState>,
    command: quicksort_ipc_contract::ExecuteOperationData,
) -> Result<String, String> {
    tracing::info!(command = "enqueue_operation", "handling");
    // Raw ExecuteOperationData — served by the legacy path and the DLL pipe;
    // the channel is the context menu, any correlation_id is generated by the
    // worker (spec #15).
    let job_id = state
        .queue
        .enqueue(
            command,
            quicksort_application::OperationSource::ContextMenu,
            None,
        )
        .map_err(|e| {
            tracing::error!(command = "enqueue_operation", error = %e, "FAIL");
            e
        })?;
    tracing::info!(command = "enqueue_operation", job_id = %job_id.id, "OK");
    Ok(job_id.id)
}

/// Enqueue a file operation (application DTO, as used by the frontend) for
/// asynchronous execution by the persistent job worker. Returns the job id.
#[tauri::command]
pub async fn enqueue_operation_v2(
    state: State<'_, AppState>,
    command: quicksort_application::OperationCommand,
) -> Result<String, String> {
    tracing::info!(command = "enqueue_operation_v2", op_type = ?command.operation_type, sources = ?command.source_paths, "handling");
    let data = crate::queue::command_to_execute_data(&command);
    // Frontend enqueues carry their origin on the job so replay keeps
    // source/correlation_id intact (spec #15, release 0.2.6).
    let job_id = state
        .queue
        .enqueue(data, command.source, Some(command.correlation_id))
        .map_err(|e| {
            tracing::error!(command = "enqueue_operation_v2", error = %e, "FAIL");
            e
        })?;
    tracing::info!(command = "enqueue_operation_v2", job_id = %job_id.id, "OK");
    Ok(job_id.id)
}

/// List all queue jobs (queued, running, completed, failed, canceled).
#[tauri::command]
pub async fn get_jobs(
    state: State<'_, AppState>,
) -> Result<Vec<quicksort_ipc_contract::JobDto>, String> {
    tracing::info!(command = "get_jobs", "handling");
    let jobs = state.queue.list();
    tracing::info!(command = "get_jobs", count = jobs.len(), "OK");
    Ok(jobs)
}

/// Cancel a queued (not yet started) job.
#[tauri::command]
pub async fn cancel_job(state: State<'_, AppState>, job_id: String) -> Result<(), String> {
    tracing::info!(command = "cancel_job", job_id = %job_id, "handling");
    let result = state.queue.cancel(&job_id).map_err(|e| e.to_string());
    match &result {
        Ok(()) => tracing::info!(command = "cancel_job", "OK"),
        Err(e) => tracing::error!(command = "cancel_job", error = %e, "FAIL"),
    }
    result
}

// ---------------------------------------------------------------------------
// System information (Dashboard, 0.2.6 feature #19 / plan Q4)
// ---------------------------------------------------------------------------

/// Snapshot of system resource metrics returned to the frontend Dashboard.
/// CPU usage is the average across all cores; temperature is read from
/// platform-specific hardware sensors when available.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct SystemInfoDto {
    /// Average CPU utilization across all cores (0.0 – 100.0).
    pub cpu_usage: f32,
    /// Number of logical CPU cores.
    pub cpu_cores: usize,
    /// Current CPU frequency in MHz (0 if unavailable).
    pub cpu_frequency_mhz: u64,
    /// CPU die temperature in °C, if the platform exposes sensor data.
    pub cpu_temperature: Option<f32>,
    /// Used RAM in bytes.
    pub ram_used_bytes: u64,
    /// Total physical RAM in bytes.
    pub ram_total_bytes: u64,
    /// Used disk space across all fixed drives in bytes.
    pub disk_used_bytes: u64,
    /// Total disk space across all fixed drives in bytes.
    pub disk_total_bytes: u64,
    /// Disk read throughput in bytes/sec since the previous poll.
    pub disk_read_bytes_per_sec: u64,
    /// Disk write throughput in bytes/sec since the previous poll.
    pub disk_write_bytes_per_sec: u64,
    /// Network receive throughput in bytes/sec since the previous poll.
    pub network_rx_bytes_per_sec: u64,
    /// Network transmit throughput in bytes/sec since the previous poll.
    pub network_tx_bytes_per_sec: u64,
}

/// Returns a snapshot of current system resource metrics for the Dashboard.
///
/// Disk/network throughput fields are computed as per-second deltas between
/// the previous and current poll using `AppState::system_sample`. On the
/// first call they are zero.
#[tauri::command]
pub async fn get_system_info(state: State<'_, AppState>) -> Result<SystemInfoDto, String> {
    let mut sys = System::new();
    sys.refresh_memory();
    // First CPU refresh — needed by sysinfo to establish a baseline.
    sys.refresh_cpu_all();
    // Small sleep so the next refresh yields a meaningful utilization value.
    std::thread::sleep(std::time::Duration::from_millis(200));
    sys.refresh_cpu_all();

    // CPU temperature — best-effort, not available on all platforms.
    let cpu_temperature = Components::new_with_refreshed_list()
        .into_iter()
        .find(|c| {
            let label = c.label().to_lowercase();
            label.contains("cpu") || label.contains("core") || label.contains("processor")
        })
        .and_then(|c| c.temperature());

    // Aggregate disk stats. `everything()` refreshes Kind + Storage + IoUsage,
    // and `usage()` exposes cumulative read/written totals.
    let mut disk_read: u64 = 0;
    let mut disk_write: u64 = 0;
    let mut disk_used: u64 = 0;
    let mut disk_total: u64 = 0;
    for disk in Disks::new_with_refreshed_list_specifics(DiskRefreshKind::everything()).list() {
        let total = disk.total_space();
        let available = disk.available_space();
        disk_total += total;
        disk_used += total - available;
        disk_read += disk.usage().total_read_bytes;
        disk_write += disk.usage().total_written_bytes;
    }

    // Aggregate network stats (cumulative totals since boot).
    let mut network_rx: u64 = 0;
    let mut network_tx: u64 = 0;
    for data in Networks::new_with_refreshed_list().list().values() {
        network_rx += data.total_received();
        network_tx += data.total_transmitted();
    }

    // Compute per-second throughput deltas against the previous sample.
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_millis() as u64);

    let mut sample = state.system_sample.lock();
    let (disk_read_bps, disk_write_bps, network_rx_bps, network_tx_bps) =
        if sample.timestamp_ms == 0 {
            // First poll — store the baseline and report zero throughput.
            *sample = SystemInfoSample {
                disk_read,
                disk_write,
                network_rx,
                network_tx,
                timestamp_ms: now_ms,
            };
            (0, 0, 0, 0)
        } else {
            let dt_ms = now_ms.saturating_sub(sample.timestamp_ms);
            let dt_secs = (dt_ms as f64) / 1000.0;
            let bps = |cur: u64, prev: u64| {
                if dt_secs > 0.0 && cur >= prev {
                    ((cur - prev) as f64 / dt_secs) as u64
                } else {
                    0
                }
            };
            let speeds = (
                bps(disk_read, sample.disk_read),
                bps(disk_write, sample.disk_write),
                bps(network_rx, sample.network_rx),
                bps(network_tx, sample.network_tx),
            );
            *sample = SystemInfoSample {
                disk_read,
                disk_write,
                network_rx,
                network_tx,
                timestamp_ms: now_ms,
            };
            speeds
        };
    drop(sample);

    let ram_total = sys.total_memory();
    let ram_used = sys.used_memory();

    let dto = SystemInfoDto {
        cpu_usage: sys.global_cpu_usage(),
        cpu_cores: sys.cpus().len(),
        cpu_frequency_mhz: sys.cpus().first().map_or(0, |c| c.frequency()),
        cpu_temperature,
        ram_used_bytes: ram_used,
        ram_total_bytes: ram_total,
        disk_used_bytes: disk_used,
        disk_total_bytes: disk_total,
        disk_read_bytes_per_sec: disk_read_bps,
        disk_write_bytes_per_sec: disk_write_bps,
        network_rx_bytes_per_sec: network_rx_bps,
        network_tx_bytes_per_sec: network_tx_bps,
    };

    tracing::debug!(
        command = "get_system_info",
        cpu = dto.cpu_usage,
        ram_pct = if ram_total > 0 {
            (ram_used as f64 / ram_total as f64 * 100.0) as f32
        } else {
            0.0
        },
        "OK"
    );
    Ok(dto)
}

// ---------------------------------------------------------------------------
// Backup commands (0.2.6 feature #20 / plan Q7)
// ---------------------------------------------------------------------------

/// Create a ZIP archive of settings.json + folders.json at the given path
/// chosen via a save dialog on the frontend.
#[tauri::command]
pub fn backup_data(target_path: String) -> Result<String, String> {
    tracing::info!(command = "backup_data", target = %target_path, "handling");
    match crate::backup::create_backup(std::path::Path::new(&target_path)) {
        Ok(path) => {
            tracing::info!(command = "backup_data", "OK — {}", path.display());
            Ok(path.display().to_string())
        }
        Err(e) => {
            tracing::error!(command = "backup_data", error = %e, "FAIL");
            Err(e)
        }
    }
}

/// Restore settings.json + folders.json from a user-selected ZIP archive.
#[tauri::command]
pub fn restore_data(backup_path: String) -> Result<String, String> {
    tracing::info!(command = "restore_data", backup = %backup_path, "handling");
    match crate::backup::restore_backup(std::path::Path::new(&backup_path)) {
        Ok(count) => {
            tracing::info!(command = "restore_data", restored = count, "OK");
            Ok(format!("Restored {count} file(s) from backup"))
        }
        Err(e) => {
            tracing::error!(command = "restore_data", error = %e, "FAIL");
            Err(e)
        }
    }
}

/// Best-effort automatic backup after a successful config change (plan Q7).
/// Failures are only logged so the originating command keeps working.
fn auto_backup_after_change() {
    match crate::backup::auto_backup() {
        Ok(path) => tracing::debug!(backup = %path.display(), "auto-backup created"),
        Err(e) => tracing::warn!(error = %e, "auto-backup failed"),
    }
}

/// Create an automatic backup in the data dir, pruning the oldest archives.
#[tauri::command]
pub fn auto_backup_now() -> Result<String, String> {
    tracing::info!(command = "auto_backup_now", "handling");
    match crate::backup::auto_backup() {
        Ok(path) => {
            tracing::info!(command = "auto_backup_now", "OK — {}", path.display());
            Ok(path.display().to_string())
        }
        Err(e) => {
            tracing::error!(command = "auto_backup_now", error = %e, "FAIL");
            Err(e)
        }
    }
}
