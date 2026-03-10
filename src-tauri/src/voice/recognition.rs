use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::io::Read;
use std::thread;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tauri::{AppHandle, Manager};

static VOICE_CAPTURE_STATE: std::sync::OnceLock<Arc<Mutex<VoiceCaptureState>>> = std::sync::OnceLock::new();

fn get_voice_state() -> &'static Arc<Mutex<VoiceCaptureState>> {
    VOICE_CAPTURE_STATE.get_or_init(|| Arc::new(Mutex::new(VoiceCaptureState::default())))
}

#[derive(Default)]
pub struct VoiceCaptureState {
    active_streams: HashMap<String, VoiceStreamHandle>,
}

pub struct VoiceStreamHandle {
    pub child: Option<std::process::Child>,
    pub stop_sender: mpsc::Sender<()>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct VoiceRecognitionConfig {
    pub url: String,
    pub api_key: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct VoiceResult {
    pub text: String,
    pub is_final: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct VoiceError {
    pub message: String,
}

#[tauri::command]
pub fn list_microphones() -> Result<Vec<String>, String> {
    let output = Command::new("arecord")
        .arg("-l")
        .output()
        .map_err(|e| format!("Failed to run arecord: {}", e))?;

    if !output.status.success() {
        return Ok(vec![]);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let microphones: Vec<String> = stdout
        .lines()
        .filter(|line| line.starts_with("card "))
        .map(|line| line.to_string())
        .collect();

    Ok(microphones)
}

#[tauri::command]
pub async fn start_voice_recognition(
    app: AppHandle,
    vosk_config: VoiceRecognitionConfig,
    silence_timeout_ms: Option<u64>,
) -> Result<String, String> {
    let stream_id = uuid::Uuid::new_v4().to_string();
    let (stop_tx, stop_rx) = mpsc::channel::<()>(1);
    let (audio_tx, _) = mpsc::channel::<Vec<u8>>(100);
    
    {
        let mut state = get_voice_state().lock().unwrap();
        state.active_streams.insert(stream_id.clone(), VoiceStreamHandle {
            child: None,
            stop_sender: stop_tx,
        });
    }

    let vosk_url = if let Some(api_key) = &vosk_config.api_key {
        format!("{}?api_key={}", vosk_config.url, api_key)
    } else {
        vosk_config.url.clone()
    };

    let stream_id_clone = stream_id.clone();
    let silence_timeout = silence_timeout_ms.unwrap_or(3000);

    tokio::spawn(async move {
        let result = run_voice_capture(&app, &stream_id_clone, &vosk_url, silence_timeout, stop_rx, audio_tx).await;
        
        let mut state = get_voice_state().lock().unwrap();
        state.active_streams.remove(&stream_id_clone);
        
        if let Err(e) = result {
            let _ = app.emit_all("voice_error", VoiceError { message: e });
        }
    });

    Ok(stream_id)
}

#[tauri::command]
pub fn stop_voice_recognition(stream_id: String) -> Result<(), String> {
    let mut state = get_voice_state().lock().unwrap();
    if let Some(handle) = state.active_streams.get_mut(&stream_id) {
        let _ = handle.stop_sender.try_send(());
        if let Some(ref mut child) = handle.child {
            let _ = child.kill();
        }
        Ok(())
    } else {
        Err("Stream not found".to_string())
    }
}

async fn run_voice_capture(
    app: &AppHandle,
    stream_id: &str,
    vosk_url: &str,
    silence_timeout_ms: u64,
    mut stop_rx: mpsc::Receiver<()>,
    _audio_tx: mpsc::Sender<Vec<u8>>,
) -> Result<(), String> {
    log::info!("Connecting to Vosk server at: {}", vosk_url);
    let (ws_stream, _) = connect_async(vosk_url)
        .await
        .map_err(|e| format!("Failed to connect to Vosk at {}: {}", vosk_url, e))?;
    
    let (mut write, mut read) = ws_stream.split();

    let init_msg = serde_json::json!({
        "config": { "sample_rate": 16000 }
    });
    write.send(Message::Text(init_msg.to_string()))
        .await
        .map_err(|e| format!("Failed to send init: {}", e))?;

    let mut child = Command::new("arecord")
        .args(["-f", "S16_LE", "-r", "16000", "-c", "1", "-t", "raw"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start arecord: {}. Ensure user has audio group permissions. Try: usermod -aG audio $USER", e))?;

    let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
    let mut stderr = child.stderr.take();

    use std::sync::mpsc as std_mpsc;
    let (std_audio_tx, std_audio_rx) = std_mpsc::channel::<Vec<u8>>();

    let audio_tx_for_thread = std_audio_tx.clone();
    thread::spawn(move || {
        if let Some(stderr) = stderr {
            let mut stderr_reader = std::io::BufReader::new(stderr);
            let mut stderr_buffer = String::new();
            if stderr_reader.read_to_string(&mut stderr_buffer).is_ok() && !stderr_buffer.is_empty() {
                eprintln!("[VoiceCapture] arecord stderr: {}", stderr_buffer);
            }
        }
    });

    thread::spawn(move || {
        let mut reader = std::io::BufReader::new(stdout);
        let mut buffer = vec![0u8; 32000];
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    if audio_tx_for_thread.send(buffer[..n].to_vec()).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });

    {
        let mut state = get_voice_state().lock().unwrap();
        if let Some(handle) = state.active_streams.get_mut(stream_id) {
            handle.child = Some(child);
        }
    }
    
    let mut last_audio_time = std::time::Instant::now();
    let silence_timeout = std::time::Duration::from_millis(silence_timeout_ms);

    loop {
        tokio::select! {
            _ = stop_rx.recv() => {
                log::info!("Voice recognition stopped by user");
                break;
            }
            _ = tokio::time::sleep(std::time::Duration::from_millis(50)) => {
                while let Ok(data) = std_audio_rx.try_recv() {
                    last_audio_time = std::time::Instant::now();
                    if write.send(Message::Binary(data)).await.is_err() {
                        break;
                    }
                }
                
                if last_audio_time.elapsed() >= silence_timeout {
                    let _ = app.emit_all("voice_result", VoiceResult { 
                        text: String::new(), 
                        is_final: true 
                    });
                    break;
                }
            }
            msg = read.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        if let Ok(result) = serde_json::from_str::<serde_json::Value>(&text) {
                            if let Some(result_arr) = result.get("result").and_then(|r| r.as_array()) {
                                if !result_arr.is_empty() {
                                    let text = result_arr.iter()
                                        .filter_map(|r| r.get("word").and_then(|w| w.as_str()))
                                        .collect::<Vec<_>>()
                                        .join(" ");
                                    let _ = app.emit_all("voice_result", VoiceResult { text, is_final: true });
                                }
                            } else if let Some(partial) = result.get("partial").and_then(|p| p.as_str()) {
                                let _ = app.emit_all("voice_result", VoiceResult { 
                                    text: partial.to_string(), 
                                    is_final: false 
                                });
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {}
                }
            }
            _ = tokio::time::sleep(silence_timeout / 2) => {
                if last_audio_time.elapsed() >= silence_timeout {
                    let _ = app.emit_all("voice_result", VoiceResult { 
                        text: String::new(), 
                        is_final: true 
                    });
                    break;
                }
            }
        }
    }

    let _ = app.emit_all("voice_end", stream_id);
    Ok(())
}
