use crate::voice::VoiceState;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::{State, AppHandle, Manager};

#[derive(Debug, Clone, PartialEq)]
pub enum VoiceWakeError {
    AsrUnavailable,
    AudioDeviceError,
    NetworkError,
    Timeout,
    Other(String),
}

impl std::fmt::Display for VoiceWakeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VoiceWakeError::AsrUnavailable => write!(f, "ASR 服务不可用"),
            VoiceWakeError::AudioDeviceError => write!(f, "音频设备错误"),
            VoiceWakeError::NetworkError => write!(f, "网络连接错误"),
            VoiceWakeError::Timeout => write!(f, "操作超时"),
            VoiceWakeError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FallbackConfig {
    pub enable_energy_only: bool,
    pub energy_threshold: f32,
    pub max_retries: u32,
    pub retry_interval_ms: u64,
}

impl Default for FallbackConfig {
    fn default() -> Self {
        Self {
            enable_energy_only: true,
            energy_threshold: 0.02,
            max_retries: 3,
            retry_interval_ms: 500,
        }
    }
}

pub struct VoiceWakeState {
    pub is_running: Arc<Mutex<bool>>,
    pub error_count: Arc<Mutex<u32>>,
    pub fallback_config: FallbackConfig,
    pub is_fallback_mode: Arc<Mutex<bool>>,
}

impl VoiceWakeState {
    pub fn new() -> Self {
        Self {
            is_running: Arc::new(Mutex::new(false)),
            error_count: Arc::new(Mutex::new(0)),
            fallback_config: FallbackConfig::default(),
            is_fallback_mode: Arc::new(Mutex::new(false)),
        }
    }
    
    pub async fn record_error(&self) -> bool {
        let mut count = self.error_count.lock().await;
        *count += 1;
        if *count >= self.fallback_config.max_retries {
            let mut fallback = self.is_fallback_mode.lock().await;
            *fallback = true;
            println!("[VoiceWake] 启用降级模式：纯能量阈值检测");
            return true;
        }
        false
    }
    
    pub async fn reset_error(&self) {
        let mut count = self.error_count.lock().await;
        *count = 0;
    }
    
    pub async fn is_in_fallback(&self) -> bool {
        *self.is_fallback_mode.lock().await
    }
}

pub fn fuzzy_match_keyword(input: &str, keyword: &str) -> bool {
    let input = input.to_lowercase().trim().to_string();
    let keyword = keyword.to_lowercase().trim().to_string();
    
    if input == keyword {
        return true;
    }
    
    if input.contains(&keyword) || keyword.contains(&input) {
        return true;
    }
    
    let pinyin_variants = vec![
        ("shine", "晒"),
        ("shine", "小晒"),
        ("小 shine", "小晒"),
        ("小 shine", "小 shai"),
        ("shine", "shai"),
    ];
    
    for (from, to) in pinyin_variants {
        let variant_keyword = keyword.replace(from, to);
        if input == variant_keyword || input.contains(&variant_keyword) {
            return true;
        }
    }
    
    let distance = levenshtein_distance(&input, &keyword);
    if distance <= 2 && input.len() > 2 && keyword.len() > 2 {
        return true;
    }
    
    false
}

fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();
    
    let s1_len = s1_chars.len();
    let s2_len = s2_chars.len();
    
    if s1_len == 0 { return s2_len; }
    if s2_len == 0 { return s1_len; }
    
    let mut dp = vec![vec![0; s2_len + 1]; s1_len + 1];
    
    for i in 0..=s1_len { dp[i][0] = i; }
    for j in 0..=s2_len { dp[0][j] = j; }
    
    for i in 1..=s1_len {
        for j in 1..=s2_len {
            let cost = if s1_chars[i - 1] == s2_chars[j - 1] { 0 } else { 1 };
            dp[i][j] = (dp[i - 1][j] + 1)
                .min(dp[i][j - 1] + 1)
                .min(dp[i - 1][j - 1] + cost);
        }
    }
    
    dp[s1_len][s2_len]
}

#[derive(Debug, Clone, PartialEq)]
pub enum WakeLoopState {
    Idle,
    Waking,
    Listening,
    Processing,
}

#[tauri::command]
pub async fn start_voice_wake(
    state: State<'_, VoiceWakeState>,
    app: AppHandle,
) -> Result<(), String> {
    let mut is_running = state.is_running.lock().await;
    if *is_running {
        return Ok(());
    }
    *is_running = true;
    
    let config = crate::config::get_app_config().map_err(|e| e.to_string())?;
    let vosk_url = config.vosk.url.clone();
    let vosk_api_key = config.vosk.api_key.clone();
    let wake_word = config.voice_wake.wake_word.clone();
    let wake_sounds = config.voice_wake.wake_sounds.clone();
    let silence_timeout = config.voice_wake.silence_timeout;
    let end_words = config.voice_wake.end_words.clone();
    
    let is_running_clone = Arc::clone(&state.is_running);
    
    tokio::spawn(async move {
        run_wake_loop(
            is_running_clone,
            app,
            wake_word,
            wake_sounds,
            silence_timeout,
            vosk_url,
            vosk_api_key,
            end_words,
        ).await;
    });
    
    Ok(())
}

#[tauri::command]
pub async fn stop_voice_wake(state: State<'_, VoiceWakeState>) -> Result<(), String> {
    let mut is_running = state.is_running.lock().await;
    *is_running = false;
    Ok(())
}

#[tauri::command]
pub async fn focus_window(app: AppHandle) -> Result<(), String> {
    let window = app.get_window("main").ok_or("Main window not found")?;
    if let Err(e) = window.show() { eprintln!("[VoiceWake] Failed to show window: {}", e); }
    if let Err(e) = window.set_focus() { eprintln!("[VoiceWake] Failed to set focus: {}", e); }
    if let Err(e) = window.unminimize() { eprintln!("[VoiceWake] Failed to unminimize window: {}", e); }
    Ok(())
}

#[tauri::command]
pub async fn get_voice_wake_status(state: State<'_, VoiceWakeState>) -> Result<serde_json::Value, String> {
    use serde_json::json;
    
    let is_running = *state.is_running.lock().await;
    let error_count = *state.error_count.lock().await;
    let is_fallback = *state.is_fallback_mode.lock().await;
    
    Ok(json!({
        "is_running": is_running,
        "error_count": error_count,
        "is_fallback_mode": is_fallback,
        "fallback_config": {
            "enable_energy_only": state.fallback_config.enable_energy_only,
            "energy_threshold": state.fallback_config.energy_threshold,
            "max_retries": state.fallback_config.max_retries,
        }
    }))
}

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use std::time::{Duration, Instant};
use crate::voice::tts_player::TtsPlayer;
use std::thread;

async fn run_wake_loop(
    is_running: Arc<Mutex<bool>>,
    app: AppHandle,
    wake_word: String,
    wake_sounds: Vec<String>,
    silence_timeout_ms: u32,
    vosk_url: String,
    vosk_api_key: String,
    end_words: Vec<String>,
) {
    println!("[VoiceWake] Starting wake loop with wake_word='{}', vosk_url={}", wake_word, vosk_url);
    
    let mut state = WakeLoopState::Idle;
    let mut last_audio_time = Instant::now();
    let energy_threshold = 0.02f32;
    let silence_timeout = Duration::from_millis(silence_timeout_ms as u64);
    
    let tts_player = TtsPlayer::new(wake_sounds.clone());
    
    let resource_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .map(|p| p.join("resources").join("tts"));
    
    let tts_player = if let Some(ref dir) = resource_dir {
        tts_player.with_resource_dir(dir.clone())
    } else {
        tts_player
    };
    
    let (audio_tx, mut audio_rx) = mpsc::channel::<Vec<f32>>(100);
    
    let audio_capture_result: Result<(), String> = (|| {
        let mut audio_capture = crate::voice::AudioCapture::new()?;
        audio_capture.start(16000, 1, audio_tx.clone())?;
        Ok(())
    })();
    
    if let Err(e) = audio_capture_result {
        eprintln!("[VoiceWake] Failed to start audio capture: {}", e);
        let _ = app.emit_all("voice-error", serde_json::json!({"message": format!("音频捕获启动失败：{}", e)}));
        return;
    }
    
    println!("[VoiceWake] Audio capture started");
    
    let mut audio_buffer: VecDeque<Vec<f32>> = VecDeque::with_capacity(10);
    const BUFFER_SIZE: usize = 10;
    
    let mut ws_write = None;
    let mut ws_read = None;
    
    while *is_running.lock().await {
        match state {
            WakeLoopState::Idle => {
                tokio::time::sleep(Duration::from_millis(50)).await;
                
                while let Ok(audio_data) = audio_rx.try_recv() {
                    last_audio_time = Instant::now();
                    
                    let energy: f32 = audio_data.iter().map(|&s| s * s).sum::<f32>() / audio_data.len() as f32;
                    
                    if energy > energy_threshold {
                        audio_buffer.push_back(audio_data.clone());
                        if audio_buffer.len() > BUFFER_SIZE {
                            audio_buffer.pop_front();
                        }
                        
                        println!("[VoiceWake] Voice activity detected (energy={:.4})", energy);
                        
                        state = WakeLoopState::Waking;
                        break;
                    }
                }
                
                if last_audio_time.elapsed() > Duration::from_secs(60) {
                    println!("[VoiceWake] Long silence detected, reinitializing");
                    last_audio_time = Instant::now();
                }
            }
            
            WakeLoopState::Waking => {
                println!("[VoiceWake] WAKING state - sending events and playing TTS");
                
                let _ = app.emit_all("voice-waked", ());
                let _ = app.emit_all("voice-state-changed", serde_json::json!({"state": "waking"}));
                
                match tts_player.play_wake_response() {
                    Ok(_) => println!("[VoiceWake] TTS response played"),
                    Err(e) => eprintln!("[VoiceWake] TTS playback error: {}", e),
                }
                
                state = WakeLoopState::Listening;
                audio_buffer.clear();
            }
            
            WakeLoopState::Listening => {
                println!("[VoiceWake] LISTENING state - connecting to ASR");
                
                let _ = app.emit_all("voice-state-changed", serde_json::json!({"state": "listening"}));
                
                let asr_url = if vosk_api_key.is_empty() {
                    vosk_url.clone()
                } else {
                    format!("{}?api_key={}", vosk_url, vosk_api_key)
                };
                
                match connect_async(&asr_url).await {
                    Ok((ws_stream, _)) => {
                        let (write, read) = ws_stream.split();
                        let mut write_pinned = Box::pin(write);
                        
                        let config = serde_json::json!({
                            "config": { "sample_rate": 16000 }
                        });
                        if let Err(e) = write_pinned.send(Message::Text(config.to_string())).await {
                            eprintln!("[VoiceWake] Failed to send ASR config: {}", e);
                            state = WakeLoopState::Idle;
                            continue;
                        }
                        
                        ws_write = Some(write_pinned);
                        ws_read = Some(Box::pin(read));
                        
                        println!("[VoiceWake] Connected to ASR service");
                        state = WakeLoopState::Processing;
                    }
                    Err(e) => {
                        eprintln!("[VoiceWake] Failed to connect to ASR: {}", e);
                        let _ = app.emit_all("voice-error", serde_json::json!({"message": format!("ASR 连接失败：{}", e)}));
                        state = WakeLoopState::Idle;
                    }
                }
            }
            
            WakeLoopState::Processing => {
                println!("[VoiceWake] PROCESSING state - receiving ASR results");
                
                let ws_write_ref = &mut ws_write;
                let ws_read_ref = &mut ws_read;
                
                tokio::select! {
                    _ = async {
                        while let Ok(audio_data) = audio_rx.try_recv() {
                            last_audio_time = Instant::now();
                            
                            if let Some(ref mut write) = ws_write_ref {
                                let bytes: Vec<u8> = audio_data
                                    .iter()
                                    .flat_map(|&s| ((s * 32767.0) as i16).to_le_bytes())
                                    .collect();
                                if let Err(e) = write.send(Message::Binary(bytes)).await {
                                    eprintln!("[VoiceWake] Failed to send audio to ASR: {}", e);
                                    break;
                                }
                            }
                        }
                    } => {}
                    
                    result = async {
                        if let Some(ref mut read) = ws_read_ref {
                            while let Some(msg) = read.next().await {
                                match msg {
                                    Ok(Message::Text(text)) => {
                                        if let Ok(result) = serde_json::from_str::<serde_json::Value>(&text) {
                                            if let Some(partial) = result.get("partial").and_then(|p| p.as_str()) {
                                                if !partial.is_empty() {
                                                    println!("[VoiceWake] Partial result: {}", partial);
                                                    let _ = app.emit_all("voice-result", serde_json::json!({
                                                        "text": partial,
                                                        "is_final": false
                                                    }));
                                                }
                                            } else if let Some(result_text) = result.get("text").and_then(|t| t.as_str()) {
                                                if !result_text.is_empty() {
                                                    println!("[VoiceWake] Final result: {}", result_text);
                                                    let _ = app.emit_all("voice-result", serde_json::json!({
                                                        "text": result_text,
                                                        "is_final": true
                                                    }));
                                                    
                                                    for end_word in &end_words {
                                                        if result_text.contains(end_word) {
                                                            println!("[VoiceWake] End word '{}' detected", end_word);
                                                            return Some("end_word");
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    Ok(Message::Close(_)) => {
                                        println!("[VoiceWake] ASR connection closed");
                                        return Some("closed");
                                    }
                                    Err(e) => {
                                        eprintln!("[VoiceWake] ASR error: {}", e);
                                        return Some("error");
                                    }
                                    _ => {}
                                }
                            }
                        }
                        None
                    } => {
                        if let Some(reason) = result {
                            println!("[VoiceWake] Processing ended: {}", reason);
                            state = WakeLoopState::Idle;
                            ws_write = None;
                            ws_read = None;
                            continue;
                        }
                    }
                    
                    _ = tokio::time::sleep(silence_timeout) => {
                        if last_audio_time.elapsed() >= silence_timeout {
                            println!("[VoiceWake] Silence timeout ({:?})", silence_timeout);
                            let _ = app.emit_all("voice-result", serde_json::json!({
                                "text": "",
                                "is_final": true
                            }));
                            state = WakeLoopState::Idle;
                            ws_write = None;
                            ws_read = None;
                        }
                    }
                }
            }
        }
    }
    
    println!("[VoiceWake] Wake loop stopped");
}

#[tauri::command]
pub async fn test_voice_wake_detection() -> Result<(), String> {
    Ok(())
}
