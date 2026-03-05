// 根据操作系统选择子系统
#[cfg(target_os = "windows")]
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
    // 获取资源目录路径
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_default();
    
    let resources_dir = exe_dir.join("resources").join("openclaw");
    let node_dir = resources_dir.join("node");
    let openclaw_dir = resources_dir.join("openclaw");
    
    // 检查并启动 OpenClaw
    if resources_dir.exists() && !openclaw::check_openclaw_running() {
        println!("[Shine Helper] Starting OpenClaw...");
        if let Err(e) = openclaw::start_openclaw_process(
            openclaw_dir.to_str().unwrap_or(""),
            node_dir.to_str().unwrap_or("")
        ) {
            eprintln!("[Shine Helper] Warning: Failed to start OpenClaw: {}", e);
        }
    }
    
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