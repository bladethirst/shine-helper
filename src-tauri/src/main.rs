#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod db;
mod openclaw;
mod commands;

use commands::{AppState, create_session, list_sessions, get_messages, delete_session, add_message, send_message};
use db::Database;
use std::sync::Mutex;

fn main() {
    let db = Database::new().expect("Failed to initialize database");
    
    tauri::Builder::default()
        .manage(AppState { db: Mutex::new(db) })
        .invoke_handler(tauri::generate_handler![
            create_session,
            list_sessions,
            get_messages,
            delete_session,
            add_message,
            send_message
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
