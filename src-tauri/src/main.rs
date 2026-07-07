//! Main entry point for Tauri application.
//! This file integrates the legacy code with the new Clean Architecture.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod folder;
mod models;
mod move_engine;
mod logging;
mod pending;
mod state;
mod activity_log;

use clap::{Parser, Subcommand};
use commands::{folders, move_cmd, settings};
use state::AppState;
use parking_lot::Mutex;
use folder::repository::JsonRepository;
use folder::service::FolderService;
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::TrayIconBuilder,
    Manager,
};


// ============================================================
// New architecture imports
// ============================================================
use std::sync::Arc;
use std::path::PathBuf;

use quicksort_application::{ExecuteOperation, UndoOperation, GetFolders, ManageFolders};
use quicksort_application::dtos::{OperationCommand, OperationResult, OverwritePolicy};
use quicksort_application::errors::UseCaseError;

use quicksort_application::ports::inbound::ApplicationFacade;
use quicksort_application::use_cases::{
    ExecuteOperationUseCase, UndoOperationUseCase,
    GetFoldersUseCase, ManageFoldersUseCase,
};

use quicksort_application::ports::outbound::IdGenerator;

use quicksort_infrastructure::{
    JsonConfigurationRepository, StdFileSystem,
    UuidGenerator, SystemClock, DefaultConflictResolver,
};
use quicksort_infrastructure::repository::InMemoryOperationRepository;

// ============================================================
// Application facade that holds all use cases and the ID generator
// ============================================================
struct AppFacade {
    execute: Arc<ExecuteOperationUseCase>,
    undo: Arc<UndoOperationUseCase>,
    get_folders: Arc<GetFoldersUseCase>,
    manage: Arc<ManageFoldersUseCase>,
    id_generator: Arc<dyn IdGenerator>,
}

impl AppFacade {
    async fn execute_operation(&self, command: OperationCommand) -> Result<OperationResult, UseCaseError> {
        ExecuteOperation::execute(&*self.execute, command).await
    }

    async fn undo_operation(&self, operation_id: String) -> Result<OperationResult, UseCaseError> {
        let id = quicksort_domain::OperationId::from_string(operation_id);
        UndoOperation::undo(&*self.undo, id).await
    }

    async fn get_folders(&self) -> Result<Vec<quicksort_domain::Folder>, UseCaseError> {
        GetFolders::get_all(&*self.get_folders).await
    }

    async fn add_folder(&self, name: String, path: String) -> Result<(), UseCaseError> {
        let id = self.id_generator.generate();
        let folder = quicksort_domain::Folder::new(
            quicksort_domain::FolderId::from_string(id),
            name,
            quicksort_domain::WindowsPath::new(&path)
                .map_err(|_| UseCaseError::InvalidCommand("Invalid path".to_string()))?,
        );
        ManageFolders::add_folder(&*self.manage, folder).await
    }

    async fn remove_folder(&self, id: String) -> Result<(), UseCaseError> {
        let folder_id = quicksort_domain::FolderId::from_string(id);
        ManageFolders::remove_folder(&*self.manage, folder_id).await
    }

    async fn toggle_favorite(&self, id: String, order: i32) -> Result<(), UseCaseError> {
        let folder_id = quicksort_domain::FolderId::from_string(id);
        ManageFolders::toggle_favorite(&*self.manage, folder_id, order).await
    }
}

// ============================================================
// Tauri command wrappers
// ============================================================
#[tauri::command]
async fn execute_operation_v2(
    state: tauri::State<'_, Arc<AppFacade>>,
    command: OperationCommand,
) -> Result<OperationResult, String> {
    state.execute_operation(command).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn undo_operation_v2(
    state: tauri::State<'_, Arc<AppFacade>>,
    operation_id: String,
) -> Result<OperationResult, String> {
    state.undo_operation(operation_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_folders_v2(
    state: tauri::State<'_, Arc<AppFacade>>,
) -> Result<Vec<quicksort_domain::Folder>, String> {
    state.get_folders().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn add_folder_v2(
    state: tauri::State<'_, Arc<AppFacade>>,
    name: String,
    path: String,
) -> Result<(), String> {
    state.add_folder(name, path).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn remove_folder_v2(
    state: tauri::State<'_, Arc<AppFacade>>,
    id: String,
) -> Result<(), String> {
    state.remove_folder(id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn toggle_favorite_v2(
    state: tauri::State<'_, Arc<AppFacade>>,
    id: String,
    order: i32,
) -> Result<(), String> {
    state.toggle_favorite(id, order).await.map_err(|e| e.to_string())
}

// ============================================================
// CLI and legacy code
// ============================================================
#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Move { target: String, file: String },
    SelectFolder { file: String },
}

fn main() {
    logging::init();
    let cli = Cli::parse();

    if let Some(cmd) = &cli.command {
        match cmd {
            Commands::Move { target, file } => {
                if let Err(e) = crate::move_engine::MoveEngine::move_file(
                    std::path::Path::new(file),
                    std::path::Path::new(target),
                ) {
                    tracing::error!("Move failed: {}", e);
                }
                return;
            }
            Commands::SelectFolder { file } => {
                crate::pending::set_pending_file(file.clone());
                start_tauri();
                return;
            }
        }
    }
    start_tauri();
}

fn start_tauri() {
    // ============================================================
    // Legacy state
    // ============================================================
    let logs = activity_log::load_logs();
    let repo = JsonRepository::new().expect("repo");
    let service = FolderService::new(repo);
    let _folders = service.list().unwrap_or_default();

    let state = AppState {
        service,
        logs: Mutex::new(logs),
    };

    // ============================================================
    // New architecture infrastructure
    // ============================================================
    let config_dir = dirs::config_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("QuickSort");
    std::fs::create_dir_all(&config_dir).unwrap_or(());
    let config_path = config_dir.join("folders.json");

    let config_repo = Arc::new(JsonConfigurationRepository::new(config_path));
    let operation_repo = Arc::new(InMemoryOperationRepository::new());
    let file_system = Arc::new(StdFileSystem::new());
    let id_generator = Arc::new(UuidGenerator);
    let clock = Arc::new(SystemClock);
    let conflict_resolver = Arc::new(DefaultConflictResolver);

    let execute_use_case = Arc::new(ExecuteOperationUseCase::new(
        config_repo.clone(),
        operation_repo.clone(),
        file_system.clone(),
        id_generator.clone(),
        clock.clone(),
        conflict_resolver.clone(),
    ));
    let undo_use_case = Arc::new(UndoOperationUseCase::new(
        operation_repo.clone(),
        file_system.clone(),
        clock.clone(),
    ));
    let get_folders_use_case = Arc::new(GetFoldersUseCase::new(config_repo.clone()));
    let manage_folders_use_case = Arc::new(ManageFoldersUseCase::new(config_repo.clone()));

    let app_facade = Arc::new(AppFacade {
        execute: execute_use_case,
        undo: undo_use_case,
        get_folders: get_folders_use_case,
        manage: manage_folders_use_case,
        id_generator: id_generator.clone(),
    });

    // ============================================================
    // Tauri builder
    // ============================================================
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(state)
        .manage(app_facade)
        .invoke_handler(tauri::generate_handler![
            // Legacy commands
            // folders::get_folders,
            // folders::update_folders,
            // folders::toggle_favorite,
            // move_cmd::move_file,
            // settings::get_mode,
            // settings::get_pending_file,
            // settings::check_menu_status,
            // settings::get_logs,
            // settings::register_com_server,
            // settings::unregister_com_server,
            // New V2 commands
            execute_operation_v2,
            undo_operation_v2,
            get_folders_v2,
            add_folder_v2,
            remove_folder_v2,
            toggle_favorite_v2,
        ])
        .setup(|app| {
            let open = MenuItemBuilder::with_id("open", "Открыть редактор").build(app)?;
            let quit = MenuItemBuilder::with_id("quit", "Выход").build(app)?;
            let menu = MenuBuilder::new(app).item(&open).separator().item(&quit).build()?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(|app, event| {
                    match event.id().as_ref() {
                        "open" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
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