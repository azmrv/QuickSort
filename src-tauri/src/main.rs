#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod config;
mod context_menu;

use clap::{Parser, Subcommand};
use anyhow::Context;
use commands::{check_menu_status, get_folders, update_folders, AppState};
use std::sync::Mutex;
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder, MenuEvent},
    tray::TrayIconBuilder,
    AppHandle, Manager,
};

#[derive(Parser)]
#[command(name = "QuickSort")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Move {
        #[arg(long)]
        target: String,
        #[arg(long)]
        file: String,
    },
}

fn main() {
    let cli = Cli::parse();

    if let Some(Commands::Move { target, file }) = cli.command {
        if let Err(e) = handle_move(&target, &file) {
            eprintln!("Ошибка перемещения: {}", e);
        }
        return;
    }

    // ---- Инициализация при первом запуске GUI ----
    let config_path = config::get_config_path();
    println!("Путь к конфигу: {:?}", config_path);
    if !config_path.exists() {
        println!("Файла нет, создаю...");
        config::save_folders(&[]).expect("Не удалось создать начальный конфиг");
        println!("Файл создан: {:?}", config_path);
    }

    let initial_folders = config::load_folders();
    let exe_path = std::env::current_exe()
        .expect("Не могу получить путь к exe")
        .to_string_lossy()
        .to_string();

    // Всегда синхронизируем меню при старте (пустой список = удалить меню)
    context_menu::ContextMenuBuilder::new(&exe_path, &initial_folders)
        .build()
        .expect("Не удалось обновить контекстное меню при старте");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState {
            folders: Mutex::new(initial_folders),
            exe_path,
        })
        .invoke_handler(tauri::generate_handler![
            get_folders,
            update_folders,
            check_menu_status
        ])
        .setup(|app| {
            let open = MenuItemBuilder::with_id("open", "Открыть редактор").build(app)?;
            let quit = MenuItemBuilder::with_id("quit", "Выход").build(app)?;
            let menu = MenuBuilder::new(app)
                .item(&open)
                .separator()
                .item(&quit)
                .build()?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(handle_tray_event)
                .build(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn handle_tray_event(app: &AppHandle, event: MenuEvent) {
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
}

fn handle_move(target: &str, file: &str) -> anyhow::Result<()> {
    let source = std::path::Path::new(file);
    let dest = std::path::Path::new(target)
        .join(source.file_name().context("Неверное имя файла")?);
    std::fs::rename(source, &dest)?;
    Ok(())
}