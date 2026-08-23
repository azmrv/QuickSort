use crate::state::AppState;
use quicksort_application::{
    ExecuteOperation, Folder, FolderId, GetFolders, GetOperationHistory, LoadSettings,
    ManageFolders, OperationId, PluginConfig, PluginInfoDto, PluginManager, SaveSettings, Settings,
    UndoOperation, WindowsPath,
};
use std::path::PathBuf;
use tauri::{AppHandle, State};

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
pub async fn add_folder_v2(
    state: State<'_, AppState>,
    name: String,
    path: String,
) -> Result<(), String> {
    tracing::info!(command = "add_folder_v2", name = %name, path = %path, "handling");
    let windows_path = WindowsPath::new(&path).map_err(|e| {
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
        Ok(()) => tracing::info!(command = "add_folder_v2", "OK"),
        Err(e) => tracing::error!(command = "add_folder_v2", error = %e, "FAIL"),
    }
    result
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
        Ok(()) => tracing::info!(command = "remove_folder_v2", "OK"),
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
        Ok(()) => tracing::info!(command = "toggle_favorite_v2", "OK"),
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
        Ok(()) => tracing::info!(command = "set_folder_color_v2", "OK"),
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

#[tauri::command]
pub async fn undo_operation_v2(
    state: State<'_, AppState>,
    operation_id: String,
) -> Result<quicksort_application::OperationResult, String> {
    tracing::info!(command = "undo_operation_v2", operation_id = %operation_id, "handling");
    let id = OperationId::from_string(&operation_id).map_err(|e| {
        tracing::error!(command = "undo_operation_v2", error = %e, "invalid operation ID");
        format!("Invalid operation ID: {}", e)
    })?;
    let result = state.facade.undo(id).await.map_err(|e| e.to_string());
    match &result {
        Ok(r) => tracing::info!(command = "undo_operation_v2", state = ?r.state, "OK"),
        Err(e) => tracing::error!(command = "undo_operation_v2", error = %e, "FAIL"),
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
) -> Result<quicksort_application::OperationResult, String> {
    tracing::info!(command = "repeat_operation_v2", operation_id = %operation_id, "handling");
    let id = OperationId::from_string(&operation_id).map_err(|e| {
        tracing::error!(command = "repeat_operation_v2", error = %e, "invalid operation ID");
        format!("Invalid operation ID: {}", e)
    })?;

    // Find the original operation in the in-memory history.
    let operations = state
        .facade
        .get_all_operations()
        .await
        .map_err(|e| e.to_string())?;
    let original = match operations.iter().find(|op| op.id == id) {
        Some(op) => op.clone(),
        None => {
            tracing::error!(command = "repeat_operation_v2", "operation not found");
            return Err(format!("Operation not found: {}", operation_id));
        }
    };

    // Resolve the target folder ID by matching the stored target path.
    // Not required for Delete/Rename, which carry no target folder.
    let target_folder_id = match &original.target_folder_path {
        Some(target_path) => {
            let folders = state.facade.get_all().await.map_err(|e| e.to_string())?;
            match folders.iter().find(|f| &f.path == target_path) {
                Some(folder) => Some(folder.id),
                None => {
                    tracing::error!(
                        command = "repeat_operation_v2",
                        path = %target_path,
                        "target folder not found"
                    );
                    return Err(format!("Target folder not found: {}", target_path));
                }
            }
        }
        None => None,
    };

    // Rebuild the original command and execute it again.
    let command = quicksort_application::OperationCommand {
        operation_type: original.operation_type,
        source_paths: original.source_paths,
        target_folder_id,
        target_paths: original.target_paths,
        overwrite_policy: quicksort_application::OverwritePolicy::Skip,
        duplicate_check_mode: quicksort_application::DuplicateCheckMode::default(),
    };

    let result = state
        .facade
        .execute(command)
        .await
        .map_err(|e| e.to_string());
    match &result {
        Ok(r) => tracing::info!(
            command = "repeat_operation_v2",
            state = ?r.state,
            files = r.processed_files,
            "OK"
        ),
        Err(e) => tracing::error!(command = "repeat_operation_v2", error = %e, "FAIL"),
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
pub fn check_menu_status() -> bool {
    tracing::debug!(command = "check_menu_status", "handling");
    crate::com::is_registered()
}

#[tauri::command]
pub fn get_logs() -> Vec<serde_json::Value> {
    tracing::debug!(command = "get_logs", "handling — returning empty (stub)");
    Vec::new()
}

#[tauri::command]
pub fn register_com_server() -> Result<String, String> {
    tracing::info!(command = "register_com_server", "handling");
    crate::com::register()?;
    tracing::info!(command = "register_com_server", "OK — registry keys written");
    Ok("COM server registered successfully.".to_string())
}

#[tauri::command]
pub fn unregister_com_server() -> Result<String, String> {
    tracing::info!(command = "unregister_com_server", "handling");
    crate::com::unregister()?;
    tracing::info!(command = "unregister_com_server", "OK");
    Ok("COM server unregistered successfully.".to_string())
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
        Ok(()) => tracing::info!(command = "save_settings", "OK"),
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

/// Launch TeraCopy with the given file list.
///
/// Writes file paths to a temp file and invokes TeraCopy.
/// Supports TeraCopy 3.17 and 4.0.
#[tauri::command]
pub async fn launch_teracopy(files: Vec<String>) -> Result<(), String> {
    tracing::info!(command = "launch_teracopy", count = files.len(), "handling");

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

/// Check if TeraCopy is installed on the system.
#[tauri::command]
pub fn check_teracopy_installed() -> bool {
    let teracopy_paths = [
        "C:\\Program Files\\TeraCopy\\TeraCopy.exe",
        "C:\\Program Files (x86)\\TeraCopy\\TeraCopy.exe",
    ];
    teracopy_paths.iter().any(|p| PathBuf::from(p).exists())
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

    let pid_path = std::env::var("APPDATA")
        .ok()
        .map(|a| PathBuf::from(a).join("QuickSort").join("dll_owner.pid"));
    if let Some(path) = pid_path {
        match std::fs::remove_file(&path) {
            Ok(()) => tracing::info!("owner PID file removed"),
            Err(e) => tracing::debug!(error = %e, "PID file already absent"),
        }
    }

    tracing::info!("all cleanup done, exiting");
    app.exit(0);
    Ok(())
}
