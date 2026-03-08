#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod db;
mod openclaw;
mod skills;
mod commands;
mod voice;

use commands::{
    AppState, create_session, list_sessions, get_messages, delete_session, add_message, send_message, send_message_stream,
    SkillsState, get_local_skills, install_skill, uninstall_skill,
    VoiceAppState, start_voice_wake, stop_voice_wake, set_voice_config, get_voice_config
};
use voice::{
    list_microphones, start_voice_recognition, stop_voice_recognition
};
use config::VoiceConfig;
use db::Database;
use skills::SkillsManager;
use std::sync::Mutex;
use std::sync::Arc;
use std::collections::HashMap;
use std::process::Command;
use std::net::TcpStream;
use std::time::Duration;
use std::path::PathBuf;
use tokio::sync::Mutex as TokioMutex;

fn check_openclaw_running() -> bool {
    TcpStream::connect_timeout(
        &"127.0.0.1:18789".parse().unwrap(),
        Duration::from_secs(1)
    ).is_ok()
}

fn start_openclaw_process(openclaw_dir: &str) -> Result<(), String> {
    let openclaw_path = PathBuf::from(openclaw_dir);
    
    let openclaw_mjs = openclaw_path.join("openclaw.mjs");
    let node_modules = openclaw_path.join("node_modules");
    
    if !openclaw_mjs.exists() {
        return Err("OpenClaw openclaw.mjs not found".to_string());
    }
    
    if !node_modules.exists() {
        return Err("OpenClaw node_modules not found".to_string());
    }
    
    // 使用系统 node 命令
    let node_cmd = if cfg!(target_os = "windows") {
        "node.exe"
    } else {
        "node"
    };
    
    // 设置 OpenClaw 配置目录
    let config_path = openclaw_path.join("data").join("openclaw.json");
    let mut cmd = Command::new(node_cmd);
    cmd.arg(openclaw_mjs.to_str().unwrap())
       .arg("gateway")
       .arg("run")
       .arg("--port")
       .arg("18789")
       .env("OPENCLAW_CONFIG_PATH", config_path.to_str().unwrap_or(""))
       .current_dir(&openclaw_path)
       .spawn()
       .map_err(|e| format!("Failed to start OpenClaw: {}", e))?;
    
    std::thread::sleep(Duration::from_secs(8));
    
    Ok(())
}

fn main() {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_default();
    
    let resources_dir = exe_dir.join("resources").join("openclaw");
    let openclaw_dir = resources_dir.join("openclaw");
    
    // 检查 resources/openclaw 目录是否存在（即 bundled 模式）
    let bundled_openclaw_dir = exe_dir.join("resources").join("openclaw");
    let is_bundled = bundled_openclaw_dir.exists() && bundled_openclaw_dir.join("openclaw.mjs").exists();
    
    // 如果是 bundled 模式且 OpenClaw 未运行，则启动
    if is_bundled && !check_openclaw_running() {
        println!("[Shine Helper] Starting OpenClaw...");
        if let Err(e) = start_openclaw_process(
            bundled_openclaw_dir.to_str().unwrap_or("")
        ) {
            eprintln!("[Shine Helper] Warning: Failed to start OpenClaw: {}", e);
        }
    }
    
    let db = Database::new().expect("Failed to initialize database");
    let skills_manager = SkillsManager::new();
    let voice_config = VoiceConfig::default();
    
    tauri::Builder::default()
        .manage(AppState { 
            db: Mutex::new(db),
            openclaw_sessions: Mutex::new(HashMap::new()),
        })
        .manage(SkillsState { manager: Mutex::new(skills_manager) })
        .manage(VoiceAppState { 
            state_machine: Arc::new(TokioMutex::new(None)),
            config: Arc::new(TokioMutex::new(voice_config)),
        })
        .invoke_handler(tauri::generate_handler![
            create_session,
            list_sessions,
            get_messages,
            delete_session,
            add_message,
            send_message,
            send_message_stream,
            get_local_skills,
            install_skill,
            uninstall_skill,
            list_microphones,
            start_voice_recognition,
            stop_voice_recognition,
            start_voice_wake,
            stop_voice_wake,
            set_voice_config,
            get_voice_config
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}