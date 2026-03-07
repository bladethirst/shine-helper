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
use std::process::Command;
use std::net::TcpStream;
use std::time::Duration;
use std::path::PathBuf;

fn check_openclaw_running() -> bool {
    TcpStream::connect_timeout(
        &"127.0.0.1:18789".parse().unwrap(),
        Duration::from_secs(1)
    ).is_ok()
}

#[cfg(target_os = "windows")]
fn get_node_exe(node_dir: &PathBuf) -> PathBuf {
    node_dir.join("node.exe")
}

#[cfg(not(target_os = "windows"))]
fn get_node_exe(node_dir: &PathBuf) -> PathBuf {
    node_dir.join("bin").join("node")
}

fn start_openclaw_process(openclaw_dir: &str, node_dir: &str) -> Result<(), String> {
    let openclaw_path = PathBuf::from(openclaw_dir);
    let node_path = PathBuf::from(node_dir);
    
    let gateway_js = openclaw_path.join("gateway.js");
    let data_dir = openclaw_path.join("data");
    let node_exe = get_node_exe(&node_path);
    
    if !node_exe.exists() {
        return Err(format!("Node.js executable not found: {:?}", node_exe));
    }
    
    if !gateway_js.exists() {
        return Err("OpenClaw gateway.js not found".to_string());
    }
    
    std::fs::create_dir_all(&data_dir).ok();
    
    let mut cmd = Command::new(&node_exe);
    cmd.arg(gateway_js.to_str().unwrap())
       .arg("--port")
       .arg("18789")
       .arg("--data")
       .arg(data_dir.to_str().unwrap())
       .spawn()
       .map_err(|e| format!("Failed to start OpenClaw: {}", e))?;
    
    std::thread::sleep(Duration::from_secs(5));
    
    Ok(())
}

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
    if resources_dir.exists() && !check_openclaw_running() {
        println!("[Shine Helper] Starting OpenClaw...");
        if let Err(e) = start_openclaw_process(
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