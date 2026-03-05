pub mod config;
pub mod db;
pub mod openclaw;
pub mod skills;
pub use config::*;
pub use db::*;
pub use openclaw::*;
pub use skills::*;

use std::process::Command;
use std::net::TcpStream;
use std::time::Duration;
use std::path::PathBuf;

/// 检查 OpenClaw 服务是否已运行
pub fn check_openclaw_running() -> bool {
    TcpStream::connect_timeout(
        &"127.0.0.1:18789".parse().unwrap(),
        Duration::from_secs(1)
    ).is_ok()
}

/// 启动 OpenClaw 进程
pub fn start_openclaw_process(openclaw_dir: &str, node_dir: &str) -> Result<(), String> {
    let openclaw_path = PathBuf::from(openclaw_dir);
    let node_path = PathBuf::from(node_dir);
    
    let gateway_js = openclaw_path.join("gateway.js");
    let data_dir = openclaw_path.join("data");
    let node_exe = node_path.join("node.exe");
    
    if !node_exe.exists() {
        return Err("Node.js executable not found".to_string());
    }
    
    if !gateway_js.exists() {
        return Err("OpenClaw gateway.js not found".to_string());
    }
    
    // 创建 data 目录
    std::fs::create_dir_all(&data_dir).ok();
    
    // 启动 OpenClaw
    let mut cmd = Command::new(&node_exe);
    cmd.arg(gateway_js.to_str().unwrap())
       .arg("--port")
       .arg("18789")
       .arg("--data")
       .arg(data_dir.to_str().unwrap())
       .spawn()
       .map_err(|e| format!("Failed to start OpenClaw: {}", e))?;
    
    // 等待服务就绪
    std::thread::sleep(Duration::from_secs(5));
    
    Ok(())
}