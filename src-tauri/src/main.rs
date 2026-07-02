#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod folder;
mod models;
mod move_engine;
mod context_menu;
mod pending;
mod logging;
use clap::{Parser, Subcommand};
use commands::AppState;
use parking_lot::Mutex;
use folder::repository::JsonRepository;
use folder::service::FolderService;
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::TrayIconBuilder,
    Manager,
};

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
                start_tauri(None);
                return;
            }
        }
    }
    start_tauri(None);
}

fn start_tauri(_file_to_move: Option<String>) {
    let repo = JsonRepository::new().expect("repo");
    let service = FolderService::new(repo);
    let folders = service.list().unwrap_or_default();
    let exe_path = std::env::current_exe().unwrap().to_string_lossy().to_string();

    if !folders.is_empty() {
        let model = context_menu::model::MenuModel::from_folders(&folders);
        context_menu::registry::RegistryInstaller::install(&model, &exe_path).ok();
    }

    let state = AppState {
        service,
        exe_path: Mutex::new(exe_path.clone()),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::get_folders,
            commands::update_folders,
            commands::toggle_favorite,
            commands::get_mode,
            commands::move_file,
            commands::get_pending_file,
            commands::check_menu_status,
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
                            context_menu::registry::RegistryInstaller::uninstall().ok();
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