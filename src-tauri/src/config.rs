use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Keyring error: {0}")]
    Keyring(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenClawConfig {
    pub url: String,
    pub use_local: bool,
    pub auto_start: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketConfig {
    pub url: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoskConfig {
    pub url: String,
    pub api_key: String,
    pub enabled: bool,
    pub silence_timeout: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceWakeConfig {
    pub enabled: bool,
    pub wake_word: String,
    pub wake_sounds: Vec<String>,
    pub silence_timeout: u32,
    pub end_words: Vec<String>,
    /// Vosk ASR 服务地址（与 vosk.url 共享配置，此处为冗余设计便于单独配置）
    #[serde(default = "default_vosk_url")]
    pub vosk_url: String,
    /// Vosk ASR API Key
    #[serde(default)]
    pub vosk_api_key: String,
}

fn default_vosk_url() -> String {
    "ws://192.168.150.26:2700".to_string()
}

impl Default for VoiceWakeConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            wake_word: "小 Shine".to_string(),
            wake_sounds: vec!["在呢".to_string(), "在的".to_string(), "我在".to_string(), "请说".to_string()],
            silence_timeout: 3000,
            end_words: vec!["结束".to_string(), "停止".to_string()],
            vosk_url: default_vosk_url(),
            vosk_api_key: "".to_string(),
        }
    }
}

impl Default for VoskConfig {
    fn default() -> Self {
        Self {
            url: "ws://192.168.150.26:2700".to_string(),
            api_key: "".to_string(),
            enabled: false,
            silence_timeout: 3000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppPreferences {
    pub theme: String,
    pub language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub openclaw: OpenClawConfig,
    pub market: MarketConfig,
    pub preferences: AppPreferences,
    pub vosk: VoskConfig,
    pub voice_wake: VoiceWakeConfig,
}

impl Default for OpenClawConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost:18789".to_string(),
            use_local: true,
            auto_start: true,
        }
    }
}

impl Default for MarketConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost:3001".to_string(),
            enabled: true,
        }
    }
}

impl Default for AppPreferences {
    fn default() -> Self {
        Self {
            theme: "system".to_string(),
            language: "zh-CN".to_string(),
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            openclaw: OpenClawConfig::default(),
            market: MarketConfig::default(),
            preferences: AppPreferences::default(),
            vosk: VoskConfig::default(),
            voice_wake: VoiceWakeConfig::default(),
        }
    }
}

fn get_config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("shine_helper")
        .join("config.json")
}

fn get_openclaw_config_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/root"))
        .join(".openclaw")
        .join("openclaw.json")
}

#[derive(Debug, Deserialize)]
struct OpenClawGatewayAuth {
    token: String,
}

#[derive(Debug, Deserialize)]
struct OpenClawGateway {
    auth: OpenClawGatewayAuth,
}

#[derive(Debug, Deserialize)]
struct OpenClawFileConfig {
    gateway: OpenClawGateway,
}

pub fn get_openclaw_token() -> Option<String> {
    let path = get_openclaw_config_path();
    eprintln!("[DEBUG] Loading OpenClaw token from: {:?}", path);
    if !path.exists() {
        eprintln!("[DEBUG] Config file does not exist");
        return None;
    }
    
    let content = fs::read_to_string(&path).ok()?;
    let config: OpenClawFileConfig = serde_json::from_str(&content).ok()?;
    eprintln!("[DEBUG] Token loaded successfully, length: {}", config.gateway.auth.token.len());
    Some(config.gateway.auth.token)
}

pub fn load_config() -> Result<AppConfig, ConfigError> {
    let path = get_config_path();
    if path.exists() {
        let content = fs::read_to_string(&path)?;
        let config: AppConfig = serde_json::from_str(&content)?;
        Ok(config)
    } else {
        let config = AppConfig::default();
        save_config(&config)?;
        Ok(config)
    }
}

pub fn save_config(config: &AppConfig) -> Result<(), ConfigError> {
    let path = get_config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(config)?;
    fs::write(&path, content)?;
    Ok(())
}

#[tauri::command]
pub fn get_app_config() -> Result<AppConfig, String> {
    load_config().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_app_config(config: AppConfig) -> Result<(), String> {
    save_config(&config).map_err(|e| e.to_string())
}
