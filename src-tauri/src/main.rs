#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod db;
mod openclaw;
mod skills;
mod commands;
mod voice;

use commands::{
    AppState, create_session, list_sessions, get_messages, delete_session, add_message, send_message, send_message_stream,
    SkillsState, get_local_skills, get_skills_dir, install_skill, uninstall_skill,
    VoiceWakeState, start_voice_wake, stop_voice_wake, test_voice_wake_detection, focus_window
};
use config::{get_app_config, save_app_config};
use voice::{
    list_microphones, start_voice_recognition, stop_voice_recognition
};
use db::Database;
use skills::SkillsManager;
use tauri::Manager;
use std::sync::Mutex;
use std::collections::HashMap;
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
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_default();
    
    let resources_dir = exe_dir.join("resources").join("openclaw");
    let node_dir = resources_dir.join("node");
    let openclaw_dir = resources_dir.join("openclaw");
    
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
        // Manage state
        .manage(AppState { 
            db: Mutex::new(db),
            openclaw_sessions: Mutex::new(HashMap::new()),
        })
        .manage(SkillsState { manager: Mutex::new(skills_manager) })
        .manage(VoiceWakeState::new())
        .setup(|app| {
            // Auto-start voice wake service when app starts if it's enabled in config
            let app_handle = app.handle();
            std::thread::spawn(move || {
                // Wait a little for initialization before attempting auto-start
                std::thread::sleep(std::time::Duration::from_secs(1));
                
                match get_app_config() {
                    Ok(config) => {
                        if config.voice_wake.enabled {
                            println!("[AutoVoiceWake] Auto-starting voice wake service...");
                            // Use tokio runtime to call the async start_voice_wake function
                            let rt = tokio::runtime::Runtime::new().unwrap();
                            rt.block_on(async move {
                                let state = app_handle.state::<VoiceWakeState>();
                                let result = crate::commands::voice_wake::start_voice_wake(state, app_handle.clone()).await;
                                if let Err(e) = result {
                                    eprintln!("[AutoVoiceWake] Failed to auto-start voice wake: {}", e);
                                } else {
                                    println!("[AutoVoiceWake] Successfully auto-started voice wake service");
                                }
                            });
                        } else {
                            println!("[AutoVoiceWake] Voice wake disabled in config - skipping auto-start");
                        }
                    }
                    Err(e) => {
                        eprintln!("[AutoVoiceWake] Failed to load config for auto-start: {}", e);
                    }
                }
            });
            Ok(())
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
            get_skills_dir,
            install_skill,
            uninstall_skill,
            list_microphones,
            start_voice_recognition,
            stop_voice_recognition,
            start_voice_wake,
            stop_voice_wake,
            focus_window,
            test_voice_wake_detection,
            get_app_config,
            save_app_config
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}