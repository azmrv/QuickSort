use tauri::State;
use crate::state::AppState;

#[tauri::command]
pub fn get_mode(state: State<AppState>) -> String {
    // В будущем можно динамически, сейчас — Editor
    "Editor".to_string()
}