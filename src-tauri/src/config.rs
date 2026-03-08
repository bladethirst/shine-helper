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

#[derive(Debug, Deserialize)]
struct OpenClawConfigFile {
    gateway: OpenClawGateway,
}

#[derive(Debug, Deserialize)]
struct OpenClawGateway {
    auth: OpenClawAuth,
    #[serde(default)]
    port: Option<u16>,
}

#[derive(Debug, Deserialize)]
struct OpenClawAuth {
    mode: String,
    #[serde(default)]
    token: Option<String>,
    #[serde(default)]
    password: Option<String>,
}

pub fn get_openclaw_token() -> Result<String, ConfigError> {
    let path = PathBuf::from("/home/helikui/.openclaw/openclaw.json");
    if !path.exists() {
        return Err(ConfigError::Keyring("OpenClaw config not found".to_string()));
    }
    
    let content = fs::read_to_string(&path)?;
    let config: OpenClawConfigFile = serde_json::from_str(&content)?;
    
    match config.gateway.auth.mode.as_str() {
        "token" => {
            config.gateway.auth.token
                .ok_or_else(|| ConfigError::Keyring("Token mode but no token found".to_string()))
        }
        "password" => {
            config.gateway.auth.password
                .ok_or_else(|| ConfigError::Keyring("Password mode but no password found".to_string()))
        }
        _ => Err(ConfigError::Keyring("Unknown auth mode".to_string())),
    }
}

pub fn get_openclaw_url() -> String {
    let path = PathBuf::from("/home/helikui/.openclaw/openclaw.json");
    if !path.exists() {
        return "http://localhost:18789".to_string();
    }
    
    if let Ok(content) = fs::read_to_string(&path) {
        if let Ok(config) = serde_json::from_str::<OpenClawConfigFile>(&content) {
            if let Some(port) = config.gateway.port {
                return format!("http://localhost:{}", port);
            }
        }
    }
    
    "http://localhost:18789".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenClawConfig {
    pub url: String,
    #[serde(skip_serializing, default)]
    pub token: String,
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

impl Default for VoskConfig {
    fn default() -> Self {
        Self {
            url: "ws://localhost:5000".to_string(),
            api_key: "".to_string(),
            enabled: false,
            silence_timeout: 3000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceConfig {
    pub enabled: bool,
    pub wake_word: String,
    pub wake_sounds: Vec<String>,
    pub silence_timeout: u32,
    pub end_words: Vec<String>,
    pub qwen_asr_url: String,
    pub qwen_asr_api_key: String,
}

impl Default for VoiceConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            wake_word: "小 Shine".to_string(),
            wake_sounds: vec!["在呢".to_string(), "你说".to_string(), "请讲".to_string()],
            silence_timeout: 3000,
            end_words: vec!["结束".to_string(), "停止".to_string()],
            qwen_asr_url: "ws://localhost:5000".to_string(),
            qwen_asr_api_key: "".to_string(),
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
    pub voice: VoiceConfig,
}

impl Default for OpenClawConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost:18789".to_string(),
            token: String::new(),
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
            voice: VoiceConfig::default(),
        }
    }
}

fn get_config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("shine_helper")
        .join("config.json")
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
