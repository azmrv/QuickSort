#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// ============================================================================
// Module declarations
// ============================================================================

// OLD modules that are kept for backward compatibility during migration.
// They will be removed once all their functionality is covered by the new
// architecture.
mod commands;            // Tauri commands (will be replaced by commands_v2)
mod folder;              // Legacy folder service (will be deleted)
mod models;              // Re-exports of domain types (temporary bridge)
mod move_engine;         // Deprecated; replaced by StdFileSystem in infra
mod logging;             // Tracing initialisation
mod pending;             // CLI --select-folder handler
mod state;               // AppState for Tauri
mod activity_log;        // Legacy activity log (will be replaced by domain events)
mod ipc_server;          // Named Pipe server (will be refactored to use facade)

// NEW architecture modules
// use … statements are at the top of the file, no new `mod` needed.

// ============================================================================
// Imports
// ============================================================================

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
// New architecture imports – brought in incrementally
// ============================================================
use std::sync::Arc;
use std::path::PathBuf;

use quicksort_application::{
    // The official facade that implements all inbound ports.
    ApplicationFacadeImpl,
    // Use cases (concrete types are needed for construction).
    use_cases::{
        ExecuteOperationUseCase,
        GetFoldersUseCase,
        ManageFoldersUseCase,
    },
    // DTOs and errors for the V2 commands.
    dtos::{OperationCommand, OperationResult},
    errors::UseCaseError,
};

use quicksort_infrastructure::{
    JsonConfigurationRepository,
    StdFileSystem,
    UuidGenerator,
    SystemClock,
    DefaultConflictResolver,
    repository::InMemoryOperationRepository,
};

// ============================================================================
// CLI definitions (unchanged)
// ============================================================================

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

// ============================================================================
// main – entry point
// ============================================================================

fn main() {
    // Initialise structured logging so we can see what's happening.
    logging::init();

    let cli = Cli::parse();

    // ----- CLI-only paths -----
    if let Some(cmd) = &cli.command {
        match cmd {
            Commands::Move { target, file } => {
                // OLD: MoveEngine::move_file (deprecated)
                // NEW: The CLI Move command should also go through the facade,
                // but for now we keep the legacy call to avoid scope creep.
                if let Err(e) = crate::move_engine::MoveEngine::move_file(
                    std::path::Path::new(file),
                    std::path::Path::new(target),
                ) {
                    tracing::error!("Move failed: {}", e);
                }
                return;
            }
            Commands::SelectFolder { file } => {
                // Store the file path so the React frontend can open the Selector.
                crate::pending::set_pending_file(file.clone());
                start_tauri();
                return;
            }
        }
    }

    // ----- Normal GUI startup -----
    start_tauri();
}

// ============================================================================
// start_tauri – wires everything and launches the Tauri application
// ============================================================================

fn start_tauri() {
    // ── Legacy state (kept for the old commands that are still active) ──
    let logs = activity_log::load_logs();
    let repo = JsonRepository::new().expect("Legacy JSON repository");
    let service = FolderService::new(repo);
    let _folders = service.list().unwrap_or_default();

    let legacy_state = AppState {
        service,
        logs: Mutex::new(logs),
    };

    // ── New architecture: build the real Application Facade ──
    // 1. Create infrastructure adapters (they implement the outbound ports).
    let config_dir = dirs::config_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("QuickSort");
    std::fs::create_dir_all(&config_dir).unwrap_or(());
    let config_path = config_dir.join("folders.json");

    let config_repo     = Arc::new(JsonConfigurationRepository::new(config_path));
    let operation_repo  = Arc::new(InMemoryOperationRepository::new());
    let file_system     = Arc::new(StdFileSystem::new());
    let id_generator    = Arc::new(UuidGenerator);
    let clock           = Arc::new(SystemClock);
    let conflict_resolver = Arc::new(DefaultConflictResolver);

    // 2. Create the use cases and wire them with their dependencies.
    let execute_use_case = Arc::new(ExecuteOperationUseCase::new(
        config_repo.clone(),
        operation_repo.clone(),
        file_system.clone(),
        id_generator.clone(),
        clock.clone(),
        conflict_resolver.clone(),
    ));
    let get_folders_use_case = Arc::new(GetFoldersUseCase::new(config_repo.clone()));
    let manage_folders_use_case = Arc::new(ManageFoldersUseCase::new(config_repo.clone()));

    // 3. Build the official facade – the single entry point for all adapters.
    let app_facade = Arc::new(ApplicationFacadeImpl {
        execute: execute_use_case.clone(),
        undo: unimplemented!("UndoOperationUseCase is not wired yet"),
        get_folders: get_folders_use_case.clone(),
        manage: manage_folders_use_case.clone(),
    });

    // 4. Start the Named Pipe server so the shell extension DLL can talk to us.
    //    The server receives the facade and calls it for every incoming command.
    ipc_server::start_pipe_server(Arc::clone(&app_facade) as Arc<_>);

    // ── Tauri builder ────────────────────────────────────────────────
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        // Register both the legacy state and the new facade.
        .manage(legacy_state)
        .manage(app_facade)
        .invoke_handler(tauri::generate_handler![
            // ---- Legacy commands (kept until the frontend is fully migrated) ----
            folders::get_folders,
            folders::update_folders,
            folders::toggle_favorite,
            move_cmd::move_file,
            settings::get_mode,
            settings::get_pending_file,
            settings::check_menu_status,
            settings::get_logs,
            settings::register_com_server,
            settings::unregister_com_server,

            // ---- New V2 commands (already wired to the facade) ----
            execute_operation_v2,
            get_folders_v2,
            add_folder_v2,
            remove_folder_v2,
            toggle_favorite_v2,
        ])
        .setup(|app| {
            // System tray with a simple menu.
            let open = MenuItemBuilder::with_id("open", "Open editor").build(app)?;
            let quit = MenuItemBuilder::with_id("quit", "Exit").build(app)?;
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
                        "quit" => app.exit(0),
                        _ => {}
                    }
                })
                .build(app)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            // Hide the window instead of closing – the app stays in the tray.
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                if let Some(window) = window.app_handle().get_webview_window("main") {
                    let _ = window.hide();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}