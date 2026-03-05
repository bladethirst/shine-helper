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
pub struct AppPreferences {
    pub theme: String,
    pub language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub openclaw: OpenClawConfig,
    pub market: MarketConfig,
    pub preferences: AppPreferences,
    pub vosk: VoskConfig,
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
