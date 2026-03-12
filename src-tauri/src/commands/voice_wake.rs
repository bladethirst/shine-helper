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

    // 空输入不匹配
    if input.is_empty() {
        return false;
    }

    if input == keyword {
        return true;
    }

    // 只有当 input 非空时才检查包含关系
    if input.contains(&keyword) {
        return true;
    }

    // 只有当 keyword 非空且 input 长度大于 1 时才检查反向包含
    if !input.is_empty() && !keyword.is_empty() && keyword.contains(&input) {
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

/// 将音频数据下采样到目标采样率
/// data: 输入的 f32 样本数组（范围 -1.0 到 1.0）
/// from_sample_rate: 原始采样率
/// to_sample_rate: 目标采样率
fn resample_to_16k(data: &[f32], from_sample_rate: u32, to_sample_rate: u32) -> Vec<f32> {
    let ratio = from_sample_rate as f32 / to_sample_rate as f32;
    let mut result = Vec::with_capacity((data.len() as f32 / ratio) as usize);

    let mut i = 0.0f32;
    while (i as usize) < data.len() {
        result.push(data[i as usize]);
        i += ratio;
    }

    result
}

/// 将立体声交错数据转换为单声道
/// 输入格式：[L, R, L, R, L, R, ...]
/// 输出：[(L+R)/2, (L+R)/2, ...] 单声道数据
fn stereo_to_mono(stereo: &[f32]) -> Vec<f32> {
    let mut mono = Vec::with_capacity(stereo.len() / 2);

    for chunk in stereo.chunks(2) {
        if chunk.len() == 2 {
            // 取左右声道的平均值
            mono.push((chunk[0] + chunk[1]) / 2.0);
        } else if chunk.len() == 1 {
            // 处理奇数长度的情况
            mono.push(chunk[0]);
        }
    }

    mono
}

#[derive(Debug, Clone, PartialEq)]
pub enum WakeLoopState {
    Idle,           // 待机，仅检测能量
    Verifying,      // 验证中，连接 ASR 检测是否为唤醒词
    Waking,         // 唤醒中，播放 TTS 响应
    Listening,      // 监听中，连接 ASR 识别用户指令（包含 Processing 流程）
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
    
    println!("[VoiceWake] Spawning wake loop task...");
    
    tokio::spawn(async move {
        println!("[VoiceWake] Task started, calling run_wake_loop...");
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
        println!("[VoiceWake] run_wake_loop returned");
    });
    
    println!("[VoiceWake] Spawn complete");
    
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
use std::sync::atomic::{AtomicBool, Ordering};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
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
    
    // 使用无界 channel 避免音频数据丢失
    let (audio_tx, mut audio_rx) = mpsc::unbounded_channel::<Vec<f32>>();
    
    println!("[VoiceWake] Channel created (unbounded), spawning capture thread...");
    
    let is_running_atomic = Arc::new(AtomicBool::new(true));
    let is_running_capture = Arc::clone(&is_running_atomic);
    
    // 在后台线程运行音频捕获
    let capture_handle = std::thread::spawn(move || {
        println!("[AudioCapture-Thread] Starting audio capture in std::thread...");
        
        let mut audio_capture = match crate::voice::AudioCapture::new() {
            Ok(capture) => capture,
            Err(e) => {
                eprintln!("[AudioCapture-Thread] Failed to create: {}", e);
                return;
            }
        };
        
        if let Err(e) = audio_capture.start(16000, 1, audio_tx) {
            eprintln!("[AudioCapture-Thread] Failed to start: {}", e);
            return;
        }
        
        println!("[AudioCapture-Thread] Stream started, waiting...");
        while is_running_capture.load(Ordering::SeqCst) {
            std::thread::sleep(Duration::from_millis(100));
        }
        println!("[AudioCapture-Thread] Exiting");
    });
    
    // 保持 handle 引用，防止线程被提前终止
    println!("[VoiceWake] Capture thread spawned");
    
    // 使用 mem::forget 防止 handle 被 drop，否则线程会被提前终止
    std::mem::forget(capture_handle);
    
    println!("[VoiceWake] Capture thread spawned, waiting for stream...");
    
    // 等待音频流启动 - 使用标准库 sleep 避免 tokio runtime 问题
    std::thread::sleep(Duration::from_millis(200));
    println!("[VoiceWake] Audio capture started, entering main loop");
    
    let mut state = WakeLoopState::Idle;
    let mut last_audio_time = Instant::now();
    let mut last_wake_time = Instant::now();
    const WAKE_COOLDOWN_MS: u64 = 2000; // 唤醒后 2 秒内不再触发
    
    let tts_player = TtsPlayer::new(wake_sounds.clone());
    
    // 尝试多个可能的资源目录位置
    let resource_dir = std::env::current_dir()
        .ok()
        .and_then(|p| {
            println!("[VoiceWake] Current dir: {:?}", p);
            // 尝试 1: 直接在 current_dir/resources/tts（开发模式，当 cwd 是 src-tauri 时）
            let tts_dir = p.join("resources").join("tts");
            println!("[VoiceWake] Checking TTS dir: {:?}", tts_dir);
            if tts_dir.exists() {
                println!("[VoiceWake] TTS resource dir found (dev mode): {:?}", tts_dir);
                return Some(tts_dir);
            }
            // 尝试 2: 从项目根目录查找（开发模式，当 cwd 是项目根目录时）
            let project_tts = p.join("src-tauri").join("resources").join("tts");
            println!("[VoiceWake] Checking project TTS dir: {:?}", project_tts);
            if project_tts.exists() {
                println!("[VoiceWake] TTS resource dir found (dev mode from root): {:?}", project_tts);
                return Some(project_tts);
            }
            None
        })
        .or_else(|| {
            // 尝试从 exe 位置查找（生产模式）
            std::env::current_exe()
                .ok()
                .and_then(|p| {
                    println!("[VoiceWake] Current exe: {:?}", p);
                    p.parent().map(|p| p.to_path_buf())
                })
                .and_then(|p| {
                    println!("[VoiceWake] Exe parent dir: {:?}", p);
                    // 尝试直接查找 resources/tts 目录
                    let tts_dir = p.join("resources").join("tts");
                    println!("[VoiceWake] Checking TTS dir: {:?}", tts_dir);
                    if tts_dir.exists() {
                        println!("[VoiceWake] TTS resource dir found (prod mode): {:?}", tts_dir);
                        Some(tts_dir)
                    } else {
                        println!("[VoiceWake] TTS resource dir not found, using beep sound");
                        None
                    }
                })
        });
    
    let tts_player = if let Some(ref dir) = resource_dir {
        println!("[VoiceWake] TTS resource dir: {:?}", dir);
        tts_player.with_resource_dir(dir.clone())
    } else {
        println!("[VoiceWake] TTS resource dir not found, using beep sound");
        tts_player
    };
    
    let mut audio_buffer: VecDeque<Vec<f32>> = VecDeque::with_capacity(10);
    const BUFFER_SIZE: usize = 10;
    
    let energy_threshold = 0.0001f32;
    let silence_timeout = Duration::from_millis(silence_timeout_ms as u64);
    
    let mut frame_count = 0u32;
    let mut last_log_time = Instant::now();
    
    type WsStream = tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;
    type WsSplitSink = futures_util::stream::SplitSink<WsStream, Message>;
    type WsSplitStream = futures_util::stream::SplitStream<WsStream>;

    let mut ws_write: Option<WsSplitSink> = None;
    let mut ws_read: Option<WsSplitStream> = None;

    let mut idle_iterations = 0u32;
    let mut last_recv_time = Instant::now();

    // 标记是否处于唤醒后聆听模式（Waking -> Listening -> Processing 流程）
    let mut is_wake_listening_mode = false;

    println!("[VoiceWake] About to enter main loop, checking is_running...");
    
    // 使用 AtomicBool 检查运行状态，避免 tokio Mutex 死锁
    loop {
        if !is_running_atomic.load(Ordering::SeqCst) {
            println!("[VoiceWake] is_running_atomic is false, exiting loop");
            break;
        }
        
        idle_iterations += 1;
        // if idle_iterations % 20 == 1 {
        //     println!("[VoiceWake] Loop iteration {}, state={:?}, last_recv={:?} ago", 
        //         idle_iterations, state, last_recv_time.elapsed());
        // }
        
        match state {
            WakeLoopState::Idle => {
                // 使用 try_recv 轮询接收数据
                match audio_rx.try_recv() {
                    Ok(audio_data) => {
                        frame_count += 1;
                        last_audio_time = Instant::now();
                        last_recv_time = Instant::now();

                        // 计算能量和样本统计
                        let energy: f32 = audio_data.iter().map(|&s| s * s).sum::<f32>() / audio_data.len() as f32;
                        let max_sample = audio_data.iter().cloned().fold(0.0f32, |a, b| a.max(b.abs()));
                        let avg_sample = audio_data.iter().map(|s| s.abs()).sum::<f32>() / audio_data.len() as f32;

                        // if frame_count % 50 == 1 {
                        //     println!("[VoiceWake] Received frame {}, len={}, energy={:.6}, max={:.6}, avg={:.6}",
                        //         frame_count, audio_data.len(), energy, max_sample, avg_sample);
                        // }

                        if energy > energy_threshold {
                            // 检查冷却时间
                            if last_wake_time.elapsed() < Duration::from_millis(WAKE_COOLDOWN_MS) {
                                if frame_count % 100 == 1 {
                                    println!("[VoiceWake] In cooldown, skipping (elapsed={:?})", last_wake_time.elapsed());
                                }
                            } else {
                                println!("[VoiceWake] Voice activity detected (energy={:.4}, max={:.4}), entering Verifying state", energy, max_sample);
                                last_wake_time = Instant::now();
                                state = WakeLoopState::Verifying;
                                audio_buffer.clear();
                                audio_buffer.push_back(audio_data);
                            }
                        }
                    }
                    Err(_) => {
                        // 没有数据，短暂等待后重试
                        std::thread::sleep(Duration::from_millis(1));
                    }
                }
            }

            WakeLoopState::Verifying => {
                println!("[VoiceWake] VERIFYING state - checking for wake word");

                // 收集更多音频数据
                let verify_duration = Duration::from_millis(1500); // 收集 1.5 秒音频
                let verify_start = Instant::now();

                while verify_start.elapsed() < verify_duration {
                    match audio_rx.try_recv() {
                        Ok(audio_data) => {
                            audio_buffer.push_back(audio_data);
                            last_audio_time = Instant::now();
                        }
                        Err(_) => {
                            std::thread::sleep(Duration::from_millis(10));
                        }
                    }
                }

                // 将缓冲的音频数据发送到 ASR 进行识别
                let asr_url = if vosk_api_key.is_empty() {
                    vosk_url.clone()
                } else {
                    format!("{}?api_key={}", vosk_url, vosk_api_key)
                };

                println!("[VoiceWake] Sending audio to ASR for wake word verification, URL: {}", asr_url);

                // 收集所有缓冲的音频数据
                // 注意：从 AudioCapture 接收到的是立体声交错数据 (LRLRLRLR...)
                let mut all_audio: Vec<f32> = Vec::new();
                while let Some(chunk) = audio_buffer.pop_front() {
                    all_audio.extend(chunk);
                }

                // 先将立体声合并为单声道（取左右声道的平均值），然后下采样到 16kHz
                let mono_audio = stereo_to_mono(&all_audio);
                println!("[VoiceWake] Converted to mono: {} samples", mono_audio.len());

                // 在后台线程执行 ASR 识别
                let wake_word_clone = wake_word.clone();
                let verify_result = std::thread::spawn(move || {
                    println!("[Verify-Thread] Creating tokio runtime...");

                    let rt = match tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                    {
                        Ok(rt) => {
                            println!("[Verify-Thread] Runtime created successfully");
                            rt
                        },
                        Err(e) => {
                            eprintln!("[Verify-Thread] Runtime build failed: {}", e);
                            return Err(format!("Runtime build failed: {}", e));
                        }
                    };

                    rt.block_on(async {
                        // 连接 ASR
                        let (ws_stream, _response) = match tokio::time::timeout(
                            Duration::from_secs(3),
                            connect_async(&asr_url)
                        ).await {
                            Ok(Ok(result)) => {
                                println!("[Verify-Thread] Connected to ASR");
                                result
                            },
                            Ok(Err(e)) => {
                                eprintln!("[Verify-Thread] Connection failed: {}", e);
                                return Err(format!("Connection failed: {}", e));
                            },
                            Err(_) => {
                                eprintln!("[Verify-Thread] Connection timeout");
                                return Err("Connection timeout".to_string());
                            }
                        };

                        let (mut write, mut read) = ws_stream.split();

                        // 发送配置
                        let config = serde_json::json!({
                            "config": { "sample_rate": 16000 }
                        });

                        println!("[Verify-Thread] Sending config...");
                        if let Err(e) = write.send(Message::Text(config.to_string())).await {
                            return Err(format!("Failed to send config: {}", e));
                        }

                        // 等待响应
                        match tokio::time::timeout(Duration::from_secs(2), read.next()).await {
                            Ok(Some(Ok(Message::Text(text)))) => {
                                println!("[Verify-Thread] Config response: {}", text);
                            },
                            Ok(Some(Ok(msg))) => {
                                println!("[Verify-Thread] Config response: {:?}", msg);
                            },
                            _ => {
                                println!("[Verify-Thread] No config response");
                            }
                        }

                        // 发送音频数据 - 先下采样到 16kHz
                        println!("[Verify-Thread] Sending {} audio samples (before resample)", mono_audio.len());

                        // 下采样到 16kHz (44100 -> 16000)
                        let resampled = resample_to_16k(&mono_audio, 44100, 16000);
                        println!("[Verify-Thread] After resample: {} samples", resampled.len());

                        // 转换为 16-bit PCM 字节 (f32 [-1.0, 1.0] -> i16)
                        let bytes: Vec<u8> = resampled
                            .iter()
                            .flat_map(|&s| ((s * 32767.0) as i16).to_le_bytes())
                            .collect();

                        // 打印音频数据统计
                        let max_val = resampled.iter().fold(0.0f32, |a, b| a.max(b.abs()));
                        let avg_val = resampled.iter().map(|s| s.abs()).sum::<f32>() / resampled.len() as f32;
                        println!("[Verify-Thread] Audio stats: max={:.4}, avg={:.6}, bytes={}", max_val, avg_val, bytes.len());

                        if let Err(e) = write.send(Message::Binary(bytes)).await {
                            return Err(format!("Failed to send audio: {}", e));
                        }

                        // 发送结束标志
                        let eof = serde_json::json!({
                            "eof": true
                        });
                        if let Err(e) = write.send(Message::Text(eof.to_string())).await {
                            eprintln!("[Verify-Thread] Failed to send EOF: {}", e);
                        }

                        // 接收识别结果
                        println!("[Verify-Thread] Waiting for recognition result...");
                        let mut recognized_text = String::new();

                        while let Ok(Some(msg)) = tokio::time::timeout(
                            Duration::from_secs(5),
                            read.next()
                        ).await {
                            match msg {
                                Ok(Message::Text(text)) => {
                                    println!("[Verify-Thread] ASR result: {}", text);
                                    if let Ok(result) = serde_json::from_str::<serde_json::Value>(&text) {
                                        // 检查 final result
                                        if let Some(result_arr) = result.get("result").and_then(|r| r.as_array()) {
                                            if !result_arr.is_empty() {
                                                recognized_text = result_arr.iter()
                                                    .filter_map(|r| r.get("word").and_then(|w| w.as_str()))
                                                    .collect::<Vec<_>>()
                                                    .join(" ");
                                                println!("[Verify-Thread] Final text: {}", recognized_text);
                                                break;
                                            }
                                        }
                                        // 检查 partial
                                        if let Some(partial) = result.get("partial").and_then(|p| p.as_str()) {
                                            if !partial.is_empty() {
                                                recognized_text = partial.to_string();
                                                println!("[Verify-Thread] Partial text: {}", recognized_text);
                                            }
                                        }
                                    }
                                }
                                Ok(Message::Close(_)) => break,
                                Err(e) => {
                                    eprintln!("[Verify-Thread] WebSocket error: {}", e);
                                    break;
                                }
                                _ => {}
                            }
                        }

                        println!("[Verify-Thread] Recognition complete: '{}'", recognized_text);
                        Ok(recognized_text)
                    })
                })
                .join()
                .unwrap_or(Err("Thread join failed".to_string()));

                // 检查识别结果是否包含唤醒词
                let wake_word_detected = match verify_result {
                    Ok(text) => {
                        println!("[VoiceWake] ASR recognized: '{}'", text);
                        fuzzy_match_keyword(&text, &wake_word)
                    }
                    Err(e) => {
                        eprintln!("[VoiceWake] Verification error: {}", e);
                        false
                    }
                };

                if wake_word_detected {
                    println!("[VoiceWake] WAKE WORD DETECTED! Entering Waking state");
                    state = WakeLoopState::Waking;
                } else {
                    println!("[VoiceWake] Not a wake word, returning to Idle");
                    is_wake_listening_mode = false;
                    state = WakeLoopState::Idle;
                }
            }
            
            WakeLoopState::Waking => {
                println!("[VoiceWake] WAKING state - sending events and playing TTS");

                let _ = app.emit_all("voice-waked", ());
                let _ = app.emit_all("voice-state-changed", serde_json::json!({"state": "waking"}));

                // 同步播放 TTS，等待完成
                match tts_player.play_wake_response() {
                    Ok(_) => println!("[VoiceWake] TTS response played"),
                    Err(e) => eprintln!("[VoiceWake] TTS playback error: {}", e),
                }

                // 短暂延迟后进入监听状态
                std::thread::sleep(Duration::from_millis(500));

                // 清空 audio_rx 通道中积累的旧音频数据（TTS 播放期间的内容）
                let mut flushed = 0;
                while audio_rx.try_recv().is_ok() {
                    flushed += 1;
                }
                println!("[VoiceWake] Flushed {} old audio frames from channel", flushed);

                // 标记进入唤醒后聆听模式
                is_wake_listening_mode = true;
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

                println!("[VoiceWake] ASR URL: {}", asr_url);

                // 在同一个 std::thread 中完成连接和 Processing 整个流程
                // 使用 Option 包裹 audio_rx，通过 take() 转移所有权
                let mut audio_rx_opt = Some(std::mem::replace(&mut audio_rx, mpsc::unbounded_channel().1));
                let processing_result = std::thread::spawn({
                    let app = app.clone();
                    let end_words = end_words.clone();
                    let silence_timeout = silence_timeout;
                    move || {
                        println!("[ASR-Thread] Creating tokio runtime for Listening+Processing loop...");

                        let rt = match tokio::runtime::Builder::new_current_thread()
                            .enable_all()
                            .build()
                        {
                            Ok(rt) => {
                                println!("[ASR-Thread] Runtime created successfully");
                                rt
                            },
                            Err(e) => {
                                eprintln!("[ASR-Thread] Runtime build failed: {}", e);
                                return Err(format!("Runtime build failed: {}", e));
                            }
                        };

                        let mut audio_rx = audio_rx_opt.take().unwrap();

                        // 在同一个 runtime 中执行连接和 Processing 循环
                        rt.block_on(async move {
                            println!("[ASR-Thread] Connecting to ASR: {}", asr_url);

                            // 连接
                            let (ws_stream, _response) = match tokio::time::timeout(Duration::from_secs(3), connect_async(&asr_url)).await {
                                Ok(Ok(result)) => {
                                    println!("[ASR-Thread] Connected successfully");
                                    result
                                },
                                Ok(Err(e)) => {
                                    eprintln!("[ASR-Thread] Connection failed: {}", e);
                                    return Err(format!("Connection failed: {}", e));
                                },
                                Err(_) => {
                                    eprintln!("[ASR-Thread] Connection timeout (3s)");
                                    return Err("Connection timeout (3s)".to_string());
                                }
                            };

                            use futures_util::StreamExt;
                            let (mut write, mut read) = ws_stream.split();

                            // 发送配置
                            let config = serde_json::json!({
                                "config": { "sample_rate": 16000 }
                            });
                            println!("[ASR-Config] Sending config...");
                            if let Err(e) = write.send(Message::Text(config.to_string())).await {
                                return Err(format!("Failed to send config: {}", e));
                            }
                            println!("[ASR-Config] Config sent, entering Processing loop");

                            // 进入 Processing 循环，返回最终结果
                            let mut last_audio_time = Instant::now();
                            let mut end_reason: Option<String> = None;
                            let mut audio_frames_sent = 0usize;
                            let mut audio_frames_received = 0usize;
                            let mut audio_samples_total = 0usize;

                            // 使用 tokio::select 等待音频数据或 ASR 结果
                            // 使用 futures::stream::unfold 或者简单的循环 + timeout
                            use tokio::sync::mpsc::error::TryRecvError;
                            loop {
                                // 尝试接收音频数据并发送
                                loop {
                                    match audio_rx.try_recv() {
                                        Ok(audio_data) => {
                                            audio_frames_received += 1;
                                            last_audio_time = Instant::now();

                                            // 1. 立体声转单声道
                                            let mono = stereo_to_mono(&audio_data);
                                            // 2. 重采样到 16kHz
                                            let resampled = resample_to_16k(&mono, 44100, 16000);
                                            // 3. 转换为 16-bit PCM 小端格式
                                            let bytes: Vec<u8> = resampled
                                                .iter()
                                                .flat_map(|&s| ((s * 32767.0) as i16).to_le_bytes())
                                                .collect();

                                            audio_samples_total += resampled.len();
                                            if let Err(e) = write.send(Message::Binary(bytes)).await {
                                                eprintln!("[ASR-Thread] Failed to send audio frame #{}: {}", audio_frames_received, e);
                                                end_reason = Some("send_error".to_string());
                                                break;
                                            }
                                            audio_frames_sent += 1;
                                            if audio_frames_sent % 50 == 1 {
                                                println!("[ASR-Thread] Sent {} frames ({} samples), received {}", audio_frames_sent, audio_samples_total, audio_frames_received);
                                            }
                                        }
                                        Err(TryRecvError::Empty) => {
                                            // 没有数据，继续接收 ASR 结果
                                            break;
                                        }
                                        Err(TryRecvError::Disconnected) => {
                                            // Channel closed
                                            println!("[ASR-Thread] Audio channel disconnected after {} frames", audio_frames_received);
                                            end_reason = Some("channel_closed".to_string());
                                            break;
                                        }
                                    }
                                }
                                if end_reason.is_some() {
                                    break;
                                }

                                // 接收 ASR 结果（带超时）
                                match tokio::time::timeout(Duration::from_millis(100), read.next()).await {
                                    Ok(Some(Ok(Message::Text(text)))) => {
                                        if let Ok(result) = serde_json::from_str::<serde_json::Value>(&text) {
                                            if let Some(partial) = result.get("partial").and_then(|p| p.as_str()) {
                                                if !partial.is_empty() {
                                                    println!("[ASR-Thread] Partial: {}", partial);
                                                    let _ = app.emit_all("voice-result", serde_json::json!({
                                                        "text": partial,
                                                        "is_final": false
                                                    }));
                                                }
                                            } else if let Some(result_text) = result.get("text").and_then(|t| t.as_str()) {
                                                if !result_text.is_empty() {
                                                    println!("[ASR-Thread] Final: {}", result_text);
                                                    let _ = app.emit_all("voice-result", serde_json::json!({
                                                        "text": result_text,
                                                        "is_final": true
                                                    }));

                                                    // 检查结束词
                                                    for end_word in &end_words {
                                                        if result_text.contains(end_word) {
                                                            println!("[ASR-Thread] End word '{}' detected", end_word);
                                                            end_reason = Some("end_word".to_string());
                                                            break;
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    Ok(Some(Ok(Message::Close(_)))) => {
                                        println!("[ASR-Thread] Connection closed by server");
                                        end_reason = Some("closed".to_string());
                                    }
                                    Ok(Some(Err(e))) => {
                                        eprintln!("[ASR-Thread] WebSocket error: {}", e);
                                        end_reason = Some("error".to_string());
                                    }
                                    Ok(Some(Ok(_))) => {
                                        // 其他消息类型（Binary, Ping, Pong, Frame），忽略
                                    }
                                    Ok(None) | Err(_) => {
                                        // Timeout 或 stream ended
                                    }
                                }

                                if end_reason.is_some() {
                                    break;
                                }

                                // 检查静音超时
                                if last_audio_time.elapsed() >= silence_timeout {
                                    println!("[ASR-Thread] Silence timeout (elapsed {:?})", last_audio_time.elapsed());
                                    end_reason = Some("silence_timeout".to_string());
                                    break;
                                }

                                // 短暂等待，避免 CPU 占用过高
                                std::thread::sleep(Duration::from_millis(10));
                            }

                            println!("[ASR-Thread] Processing loop ended: {:?}, sent {} frames, received {} frames", end_reason, audio_frames_sent, audio_frames_received);
                            Ok(end_reason)
                        })
                    }
                })
                .join()
                .unwrap_or(Err("Thread join failed".to_string()));

                println!("[VoiceWake] Processing result: {:?}", processing_result);

                // 发送 Processing 完成事件
                let _ = app.emit_all("voice-result", serde_json::json!({
                    "text": "",
                    "is_final": true
                }));

                // 如果是唤醒后聆听模式，发送完成事件
                if is_wake_listening_mode {
                    println!("[VoiceWake] Emitting voice-input-complete");
                    let _ = app.emit_all("voice-input-complete", ());
                    is_wake_listening_mode = false;
                }

                // 返回 Idle 状态
                state = WakeLoopState::Idle;
            }
        }
    }

    is_running_atomic.store(false, Ordering::SeqCst);
    println!("[VoiceWake] Wake loop stopped");
}

#[tauri::command]
pub async fn test_voice_wake_detection() -> Result<(), String> {
    Ok(())
}
