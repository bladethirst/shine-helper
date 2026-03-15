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
use std::process::{Command, Child};
use std::net::TcpStream;
use std::time::Duration;
use std::path::{PathBuf, Path};
use std::io::{self, Write};
use std::fs;
use std::collections::HashMap;

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

/// Start OpenClaw process with proper environment variables
fn start_openclaw_process(
    openclaw_dir: &Path,
    node_dir: &Path,
    config_path: &Path,
    state_dir: &Path,
) -> Result<Child, String> {
    let node_exe = get_node_exe(&node_dir.to_path_buf());
    let openclaw_mjs = openclaw_dir.join("openclaw.mjs");
    let data_dir = openclaw_dir.join("data");

    // Check Node.js executable
    if !node_exe.exists() {
        return Err(format!("Node.js executable not found: {:?}", node_exe));
    }

    // Check OpenClaw main file
    if !openclaw_mjs.exists() {
        return Err(format!("OpenClaw openclaw.mjs not found: {:?}", openclaw_mjs));
    }

    // Check config file
    if !config_path.exists() {
        return Err(format!("OpenClaw config not found: {:?}", config_path));
    }

    // Create data directory
    fs::create_dir_all(&data_dir)
        .map_err(|e| format!("Failed to create data directory: {}", e))?;

    println!("[Shine Helper] Starting OpenClaw gateway...");
    println!("  Node.js: {:?}", node_exe);
    println!("  OpenClaw: {:?}", openclaw_mjs);
    println!("  Config: {:?}", config_path);
    println!("  State Dir: {:?}", state_dir);

    // Build command with environment variables
    // OPENCLAW_HOME and OPENCLAW_STATE_DIR both point to state_dir (resources/.openclaw)
    // OPENCLAW_CONFIG_PATH points to config_path (resources/openclaw.json)
    let mut cmd = Command::new(&node_exe);
    cmd.env("OPENCLAW_HOME", state_dir)
       .env("OPENCLAW_STATE_DIR", state_dir)
       .env("OPENCLAW_CONFIG_PATH", config_path)
       .arg(&openclaw_mjs)
       .arg("gateway")
       .arg("run")
       .arg("--port")
       .arg("18789");

    // Redirect output to log file
    let log_path = std::env::temp_dir().join("openclaw.log");
    let log_file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&log_path)
        .map_err(|e| format!("Failed to create log file: {}", e))?;

    cmd.stdout(log_file.try_clone().map_err(|e| e.to_string())?);
    cmd.stderr(log_file);

    // Spawn the process
    let mut child = cmd.spawn()
        .map_err(|e| format!("Failed to spawn OpenClaw process: {}", e))?;

    println!("[Shine Helper] Waiting for OpenClaw to start...");
    std::thread::sleep(Duration::from_secs(8));

    // Verify service is running
    if check_openclaw_running() {
        println!("[Shine Helper] OpenClaw gateway is ready on port 18789");
        Ok(child)
    } else {
        let _ = child.kill();
        let log_content = fs::read_to_string(std::env::temp_dir().join("openclaw.log"))
            .unwrap_or_else(|_| "Cannot read log file".to_string());
        Err(format!(
            "OpenClaw failed to start. Last 20 lines of log:\n{}",
            log_content.lines().rev().take(20).collect::<Vec<_>>().join("\n")
        ))
    }
}

/// Check all prerequisites for OpenClaw
fn check_openclaw_prerequisites(
    node_dir: &Path,
    openclaw_dir: &Path,
    config_path: &Path,
) -> Result<(), String> {
    let node_exe = get_node_exe(&node_dir.to_path_buf());

    // Check Node.js
    if !node_exe.exists() {
        return Err(format!(
            "Node.js not found at {:?}\nPlease ensure resources/openclaw/node directory contains Node.js",
            node_exe
        ));
    }

    // Check OpenClaw app
    if !openclaw_dir.exists() {
        return Err(format!(
            "OpenClaw application not found at {:?}",
            openclaw_dir
        ));
    }

    // Check config
    if !config_path.exists() {
        return Err(format!(
            "OpenClaw config not found at {:?}",
            config_path
        ));
    }

    // Check openclaw.mjs
    let openclaw_mjs = openclaw_dir.join("openclaw.mjs");
    if !openclaw_mjs.exists() {
        return Err(format!(
            "openclaw.mjs not found at {:?}",
            openclaw_mjs
        ));
    }

    Ok(())
}

fn main() {
    println!("[Shine Helper] ====================================");
    println!("[Shine Helper] Starting Shine Helper...");
    println!("[Shine Helper] ====================================");

    // Get executable directory
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_default();

    println!("[Shine Helper] Executable directory: {:?}", exe_dir);

    // Set up OpenClaw directories
    // base_dir = resources directory
    // openclaw_dir = resources/openclaw (where openclaw.mjs and node_modules live)
    // node_dir = resources/node (where Node.js runtime lives)
    // state_dir = resources/.openclaw (where state and config live)
    let base_dir = exe_dir.join("resources");
    let openclaw_dir = base_dir.join("openclaw"); // OpenClaw app files
    let node_dir = base_dir.join("node"); // Node.js runtime
    let state_dir = base_dir.join(".openclaw"); // State directory (same as OPENCLAW_HOME)
    let config_path = base_dir.join("openclaw.json"); // Config file

    // Create necessary directories (state_dir = resources/.openclaw)
    if let Err(e) = fs::create_dir_all(&state_dir) {
        eprintln!("[Shine Helper] Failed to create state directory: {}", e);
    } else {
        println!("[Shine Helper] Created state directory: {:?}", state_dir);
    }

    // Variable to hold OpenClaw process
    let mut openclaw_process: Option<Child> = None;

    // Check if OpenClaw resources exist
    if openclaw_dir.exists() {
        // Check prerequisites
        match check_openclaw_prerequisites(
            &openclaw_dir,
            &node_dir,
            &config_path,
        ) {
            Ok(_) => {
                // Check if OpenClaw is already running
                if check_openclaw_running() {
                    println!("[Shine Helper] OpenClaw gateway is already running on port 18789");
                } else {
                    // Start OpenClaw - use state_dir for both OPENCLAW_HOME and OPENCLAW_STATE_DIR
                    match start_openclaw_process(
                        &openclaw_dir,
                        &node_dir,
                        &config_path,
                        &state_dir, // state_dir = resources/.openclaw
                    ) {
                        Ok(child) => {
                            openclaw_process = Some(child);
                            println!("[Shine Helper] OpenClaw gateway started successfully");
                        }
                        Err(e) => {
                            eprintln!("[Shine Helper] Warning: Failed to start OpenClaw: {}", e);
                            eprintln!("[Shine Helper] Some features may not work without OpenClaw");
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("[Shine Helper] OpenClaw prerequisites check failed: {}", e);
                eprintln!("[Shine Helper] Running in limited mode without OpenClaw");
            }
        }
    } else {
        println!("[Shine Helper] OpenClaw resources not found, running in limited mode");
        println!("[Shine Helper] Expected resources at: {:?}", openclaw_dir);
    }

    // Initialize database
    let db = Database::new().expect("Failed to initialize database");
    let skills_manager = SkillsManager::new();

    // Build and run Tauri application
    tauri::Builder::default()
        .manage(AppState {
            db: Mutex::new(db),
            openclaw_sessions: Mutex::new(HashMap::new()),
        })
        .manage(SkillsState { manager: Mutex::new(skills_manager) })
        .manage(VoiceWakeState::new())
        .setup(|app| {
            // Auto-start voice wake service when app starts if enabled
            let app_handle = app.handle();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_secs(1));

                match get_app_config() {
                    Ok(config) => {
                        if config.voice_wake.enabled {
                            println!("[AutoVoiceWake] Auto-starting voice wake service...");
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

    // Cleanup: kill OpenClaw process on exit
    if let Some(mut child) = openclaw_process {
        let _ = child.kill();
        println!("[Shine Helper] OpenClaw gateway stopped");
    }
}
