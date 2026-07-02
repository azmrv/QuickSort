use clap::{Parser, Subcommand};
use anyhow::Result;
use std::path::PathBuf;


#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Install {
        config: PathBuf,
        exe_path: String,
    },
    Uninstall,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Install { config, exe_path } => {
            let data = std::fs::read_to_string(&config)?;
            let folders: Vec<quicksort::models::Folder> = serde_json::from_str(&data)?;
            let model = quicksort::context_menu::model::MenuModel::from_folders(&folders);
            quicksort::context_menu::registry::RegistryInstaller::install(&model, &exe_path)?;
            println!("Menu installed successfully.");
        }
        Commands::Uninstall => {
            quicksort::context_menu::registry::RegistryInstaller::uninstall()?;
            println!("Menu removed successfully.");
        }
    }
    Ok(())
}