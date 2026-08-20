#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod ipc;
mod logging;
mod pending;
mod state;

use clap::{Parser, Subcommand};
use state::AppState;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::TrayIconBuilder,
    Manager,
};

use quicksort_application::{
    use_cases::{
        ExecuteOperationUseCase, GetFoldersUseCase, ManageFoldersUseCase, UndoOperationUseCase,
    },
    ApplicationFacadeImpl,
};

use quicksort_infrastructure::JsonConfigurationRepository;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    SelectFolder { file: String },
}

fn main() {
    logging::init();

    let cli = Cli::parse();

    if let Some(Commands::SelectFolder { file }) = &cli.command {
        crate::pending::set_pending_file(file.clone());
        start_tauri();
        return;
    }

    start_tauri();
}

fn start_tauri() {
    let config_dir = dirs::config_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("QuickSort");
    std::fs::create_dir_all(&config_dir).unwrap_or(());
    let config_path = config_dir.join("folders.json");

    let config_repo = Arc::new(JsonConfigurationRepository::new(config_path.clone()));

    let execute_use_case = ExecuteOperationUseCase::new(
        Box::new(quicksort_infrastructure::repository::InMemoryOperationRepository::new()),
        Box::new(quicksort_infrastructure::JsonConfigurationRepository::new(
            config_path.clone(),
        )),
        Box::new(quicksort_infrastructure::StdFileSystem::new()),
        Box::new(quicksort_infrastructure::UuidGenerator),
        Box::new(quicksort_infrastructure::SystemClock),
    );

    let get_folders_use_case = GetFoldersUseCase::new(config_repo.clone());
    let manage_folders_use_case = ManageFoldersUseCase::new(config_repo.clone());

    let undo_use_case = UndoOperationUseCase::new(
        Box::new(quicksort_infrastructure::repository::InMemoryOperationRepository::new()),
        Box::new(quicksort_infrastructure::StdFileSystem::new()),
    );

    let facade = Arc::new(ApplicationFacadeImpl::new(
        Arc::new(execute_use_case),
        Arc::new(undo_use_case),
        Arc::new(get_folders_use_case),
        Arc::new(manage_folders_use_case),
    ));

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
            commands::get_folders_v2,
            commands::add_folder_v2,
            commands::remove_folder_v2,
            commands::toggle_favorite_v2,
            commands::get_mode,
            commands::get_pending_file,
            commands::check_menu_status,
            commands::get_logs,
            commands::register_com_server,
            commands::unregister_com_server,
        ])
        .setup(|app| {
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
                    "quit" => app.exit(0),
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
