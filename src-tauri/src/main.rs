#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod db;
mod openclaw;
mod skills;
mod commands;

use commands::{
    AppState, create_session, list_sessions, get_messages, delete_session, add_message, send_message,
    SkillsState, get_local_skills, install_skill, uninstall_skill
};
use db::Database;
use skills::SkillsManager;
use std::sync::Mutex;

fn main() {
    let db = Database::new().expect("Failed to initialize database");
    let skills_manager = SkillsManager::new();
    
    tauri::Builder::default()
        .manage(AppState { db: Mutex::new(db) })
        .manage(SkillsState { manager: Mutex::new(skills_manager) })
        .invoke_handler(tauri::generate_handler![
            create_session,
            list_sessions,
            get_messages,
            delete_session,
            add_message,
            send_message,
            get_local_skills,
            install_skill,
            uninstall_skill
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
