#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod folder;
mod models;
mod move_engine;
mod context_menu;
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

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Переместить файл (позиционные аргументы: TARGET FILE)
    Move {
        target: String,
        file: String,
    },
    /// Открыть окно выбора папки (позиционный аргумент: FILE)
    SelectFolder {
        file: String,
    },
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
    let logs = activity_log::load_logs();
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
        logs: Mutex::new(logs),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            folders::get_folders,
            folders::update_folders,
            folders::toggle_favorite,
            move_cmd::move_file,
            settings::get_mode,
            settings::get_pending_file,
            settings::check_menu_status,
            settings::get_logs,
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