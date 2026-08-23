#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod com;
mod commands;
mod ipc;
mod logging;
mod metadata;
mod pending;
mod progress;
mod state;

use clap::{Parser, Subcommand};
use state::AppState;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::TrayIconBuilder,
    Emitter, Manager,
};

use quicksort_application::ports::inbound::PluginInfoDto;
use quicksort_application::{
    use_cases::{
        ExecuteOperationUseCase, GetFoldersUseCase, GetOperationHistoryUseCase,
        LoadSettingsUseCase, ManageFoldersUseCase, PluginConfigRepository, PluginLoader,
        PluginManagerUseCase, SaveSettingsUseCase, SearchFilesUseCase, UndoOperationUseCase,
    },
    ApplicationFacadeImpl, PluginConfig,
};

use quicksort_infrastructure::JsonConfigurationRepository;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Unregister the COM server and exit
    #[arg(long)]
    unregister: bool,
}

#[derive(Subcommand)]
enum Commands {
    SelectFolder { file: String },
}

use quicksort_application::errors::UseCaseError;

// ---------------------------------------------------------------------------
// Stub implementations for plugin system (no real plugins yet)
// ---------------------------------------------------------------------------

struct StubPluginLoader;

#[async_trait::async_trait]
impl PluginLoader for StubPluginLoader {
    async fn discover_plugins(&self) -> Result<Vec<PluginInfoDto>, UseCaseError> {
        Ok(vec![])
    }
    fn plugin_directory(&self) -> &std::path::Path {
        std::path::Path::new("")
    }
}

struct StubPluginConfigRepo;

#[async_trait::async_trait]
impl PluginConfigRepository for StubPluginConfigRepo {
    async fn load_config(&self, _plugin_id: &str) -> Result<PluginConfig, UseCaseError> {
        Ok(PluginConfig {
            id: String::new(),
            enabled: false,
            settings: serde_json::Value::Null,
        })
    }
    async fn save_config(
        &self,
        _plugin_id: &str,
        _config: &PluginConfig,
    ) -> Result<(), UseCaseError> {
        Ok(())
    }
    async fn is_enabled(&self, _plugin_id: &str) -> Result<bool, UseCaseError> {
        Ok(false)
    }
    async fn set_enabled(&self, _plugin_id: &str, _enabled: bool) -> Result<(), UseCaseError> {
        Ok(())
    }
}

fn main() {
    logging::init();
    tracing::info!(app = "quicksort", "starting");

    let cli = Cli::parse();

    if cli.unregister {
        tracing::info!("--unregister flag: unregistering COM server and exiting");
        match com::unregister() {
            Ok(()) => {
                tracing::info!("COM server unregistered successfully");
                println!("COM server unregistered successfully.");
            }
            Err(e) => {
                tracing::error!("Failed to unregister COM server: {}", e);
                eprintln!("Failed to unregister COM server: {}", e);
                std::process::exit(1);
            }
        }
        return;
    }

    if let Some(Commands::SelectFolder { file }) = &cli.command {
        tracing::info!(file = %file, "select-folder subcommand");
        crate::pending::set_pending_file(file.clone());
        start_tauri();
        return;
    }

    start_tauri();
}

fn ensure_dll_copied() {
    let exe_dir = match std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
    {
        Some(d) => d,
        None => {
            tracing::warn!("Cannot determine exe directory");
            return;
        }
    };

    // Verify DLL exists next to exe
    let dll = exe_dir.join("context_menu_dll.dll");
    if dll.exists() {
        tracing::debug!(path = %dll.display(), "DLL found next to exe");
    } else {
        tracing::warn!(
            path = %dll.display(),
            "DLL not found next to exe — COM registration will fail until DLL is built"
        );
    }

    // Copy quicksort.ico next to exe so the shell extension DLL can find it.
    // The DLL looks for the icon relative to its own path.
    let icon_dest = exe_dir.join("quicksort.ico");
    if !icon_dest.exists() {
        // Try CARGO_MANIFEST_DIR/../resources/quicksort.ico (build-time path)
        if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
            let icon_src = std::path::PathBuf::from(manifest_dir)
                .join("..")
                .join("resources")
                .join("quicksort.ico");
            if icon_src.exists() {
                match std::fs::copy(&icon_src, &icon_dest) {
                    Ok(_) => tracing::info!(src = %icon_src.display(), dst = %icon_dest.display(), "Copied icon next to exe"),
                    Err(e) => tracing::warn!(error = %e, "Failed to copy icon next to exe"),
                }
            } else {
                tracing::debug!(path = %icon_src.display(), "Icon source not found at build-time path, trying runtime path");
            }
        }
        // Fallback: try relative to current working directory
        if !icon_dest.exists() {
            let icon_cwd = std::path::Path::new("resources").join("quicksort.ico");
            if icon_cwd.exists() {
                match std::fs::copy(&icon_cwd, &icon_dest) {
                    Ok(_) => tracing::info!(src = %icon_cwd.display(), dst = %icon_dest.display(), "Copied icon next to exe (cwd fallback)"),
                    Err(e) => tracing::warn!(error = %e, "Failed to copy icon next to exe (cwd fallback)"),
                }
            }
        }
    }
}

fn start_tauri() {
    tracing::info!("startup");

    ensure_dll_copied();

    let config_dir = std::env::var("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("QuickSort");
    std::fs::create_dir_all(&config_dir).unwrap_or(());
    let config_path = config_dir.join("folders.json");
    let settings_path = config_dir.join("settings.json");

    let config_repo = Arc::new(JsonConfigurationRepository::new(config_path.clone()));
    let settings_repo = Arc::new(quicksort_infrastructure::JsonSettingsRepository::new(
        settings_path,
    ));

    // Shared operation repository — all use cases must reference the same instance
    // so that execute writes to the same store that history reads from.
    let operation_repo = quicksort_infrastructure::repository::InMemoryOperationRepository::new();

    let execute_use_case = ExecuteOperationUseCase::new(
        Box::new(operation_repo.clone_shared()),
        Box::new(quicksort_infrastructure::JsonConfigurationRepository::new(
            config_path.clone(),
        )),
        Box::new(quicksort_infrastructure::StdFileSystem::new()),
        Box::new(quicksort_infrastructure::UuidGenerator),
        Box::new(quicksort_infrastructure::SystemClock),
        Box::new(quicksort_infrastructure::DuplicateDetectionAdapter::new(
            quicksort_infrastructure::NameChecker,
        )),
    )
    .with_progress_reporter(Box::new(progress::TauriProgressReporter::new()));

    let get_folders_use_case = GetFoldersUseCase::new(config_repo.clone());
    let manage_folders_use_case = ManageFoldersUseCase::new(config_repo.clone());
    let load_settings_use_case = LoadSettingsUseCase::new(settings_repo.clone());
    let save_settings_use_case = SaveSettingsUseCase::new(settings_repo.clone());

    let undo_use_case = UndoOperationUseCase::new(
        Box::new(operation_repo.clone_shared()),
        Box::new(quicksort_infrastructure::StdFileSystem::new()),
    );
    let get_operation_history_use_case = GetOperationHistoryUseCase::new(Box::new(
        operation_repo.clone_shared(),
    ));

    let search_files_use_case =
        SearchFilesUseCase::new(Arc::new(quicksort_infrastructure::FsFileSearch::new()));

    let plugin_manager_use_case =
        PluginManagerUseCase::new(Arc::new(StubPluginLoader), Arc::new(StubPluginConfigRepo));

    let facade = Arc::new(
        ApplicationFacadeImpl::new(
            Arc::new(execute_use_case),
            Arc::new(undo_use_case),
            Arc::new(get_folders_use_case),
            Arc::new(manage_folders_use_case),
            Arc::new(get_operation_history_use_case),
            Arc::new(load_settings_use_case),
            Arc::new(save_settings_use_case),
        )
        .with_search_files(Arc::new(search_files_use_case))
        .with_plugin_manager(Arc::new(plugin_manager_use_case)),
    );

    let facade_for_ipc = Arc::clone(&facade);
    std::thread::Builder::new()
        .name("ipc-pipe-server".into())
        .spawn(move || {
            crate::ipc::server::start_pipe_server(facade_for_ipc);
        })
        .expect("failed to spawn IPC pipe server thread");

    let app_state = AppState { facade };

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::execute_operation_v2,
            commands::undo_operation_v2,
            commands::repeat_operation_v2,
            commands::get_folders_v2,
            commands::add_folder_v2,
            commands::remove_folder_v2,
            commands::toggle_favorite_v2,
            commands::set_folder_color_v2,
            commands::get_mode,
            commands::get_pending_file,
            commands::check_menu_status,
            commands::get_logs,
            commands::register_com_server,
            commands::unregister_com_server,
            commands::get_app_version,
            commands::get_settings,
            commands::save_settings,
            commands::get_operations,
            commands::launch_teracopy,
            commands::check_teracopy_installed,
            commands::create_new_folder,
            commands::list_plugins,
            commands::get_plugin_config,
            commands::save_plugin_config,
            commands::set_plugin_enabled,
            commands::rescan_plugins,
            commands::search_files,
            commands::get_app_metadata,
        ])
        .setup(|app| {
            logging::set_app_handle(app.handle().clone());
            progress::set_app_handle(app.handle().clone());

            let handle = app.handle().clone();
            std::thread::Builder::new()
                .name("com-register".into())
                .spawn(move || {
                    use com::RegistrationStatus;
                    let status = com::check_registration();
                    match &status {
                        RegistrationStatus::Active => {
                            tracing::info!("COM registration: {}", status);
                            let _ = handle.emit("com-status", "active");
                        }
                        RegistrationStatus::NotRegistered
                        | RegistrationStatus::PathMismatch { .. } => {
                            tracing::info!("COM registration: {} — registering", status);
                            let _ = handle.emit("com-status", "registering");
                            match com::register() {
                                Ok(()) => {
                                    tracing::info!("COM registered");
                                    let _ = handle.emit("com-status", "active");
                                }
                                Err(e) => {
                                    tracing::error!(error = %e, "COM registration failed");
                                    let _ = handle.emit("com-status", "error");
                                }
                            }
                        }
                        RegistrationStatus::DllMissing => {
                            tracing::warn!("COM registration: {} — skipping", status);
                            let _ = handle.emit("com-status", "error");
                        }
                    }
                })
                .expect("failed to spawn COM register thread");

            let open = MenuItemBuilder::with_id("open", "Open editor").build(app)?;
            let quit = MenuItemBuilder::with_id("quit", "Exit").build(app)?;
            let menu = MenuBuilder::new(app)
                .item(&open)
                .separator()
                .item(&quit)
                .build()?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "open" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => {
                        // Just exit — COM stays registered, DLL stays loaded in Explorer.
                        // Re-registration happens automatically on next app start.
                        app.exit(0);
                    }
                    _ => {}
                })
                .build(app)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                if let Some(window) = window.app_handle().get_webview_window("main") {
                    let _ = window.hide();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
