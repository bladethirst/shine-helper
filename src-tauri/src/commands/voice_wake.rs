use crate::voice::{VoiceState};
use std::sync::Arc;
use tauri::{State, AppHandle, Manager};
use tokio::sync::Mutex;
use std::thread;

pub struct VoiceWakeState {
    pub is_running: Arc<Mutex<bool>>,
}

impl VoiceWakeState {
    pub fn new() -> Self {
        Self {
            is_running: Arc::new(Mutex::new(false)),
        }
    }
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
    drop(is_running);

    let running = Arc::clone(&state.is_running);
    let app_clone = app.clone();
    
    thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            run_wake_loop(running, app_clone).await;
        });
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
    if let Err(e) = window.show() {
        eprintln!("[WindowFocus] Failed to show window: {}", e);
    }
    if let Err(e) = window.set_focus() {
        eprintln!("[WindowFocus] Failed to set focus: {}", e);
    }
    if let Err(e) = window.unminimize() {
        eprintln!("[WindowFocus] Failed to unminimize window: {}", e);
    }
    Ok(())
}

#[tauri::command]
pub async fn test_voice_wake_detection(
    app: AppHandle,
) -> Result<String, String> {
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
    use std::sync::mpsc as std_mpsc;
    
    let host = cpal::default_host();
    let device = match host.default_input_device() {
        Some(d) => {
            println!("[TestVoiceWake] Found audio input device: {:?}", d.name().unwrap_or_else(|_| "Unknown".to_string()));
            d
        },
        None => return Err("No audio input device found".to_string()),
    };

    let config = match device.default_input_config() {
        Ok(c) => {
            println!("[TestVoiceWake] Audio config: {:?}", c);
            c
        },
        Err(e) => return Err(format!("Failed to get audio config: {}", e)),
    };

    let (audio_tx, audio_rx) = std_mpsc::channel::<Vec<f32>>();
    
    let audio_tx_clone = audio_tx.clone();
    let stream = match device.build_input_stream(
        &config.into(),
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            let _ = audio_tx_clone.send(data.to_vec());
        },
        |err| eprintln!("[TestVoiceWake] Stream error: {}", err),
        None,
    ) {
        Ok(s) => s,
        Err(e) => return Err(format!("Failed to build audio stream: {}", e)),
    };

    if let Err(e) = stream.play() {
        return Err(format!("Failed to start audio stream: {}", e));
    }

    println!("[TestVoiceWake] Started audio stream for 1 second to test...");

    let test_start = std::time::Instant::now();
    let duration = std::time::Duration::from_secs(1);
    let mut energy_sum = 0.0;
    let mut sample_count = 0;

    while test_start.elapsed() < duration {
        std::thread::sleep(std::time::Duration::from_millis(100));
        
        while let Ok(audio_data) = audio_rx.try_recv() {
            let energy: f32 = audio_data.iter().map(|&s| s * s).sum::<f32>() / audio_data.len() as f32;
            energy_sum += energy;
            sample_count += 1;
        }
    }

    let average_energy = if sample_count > 0 { energy_sum / sample_count as f32 } else { 0.0 };
    
    stream.pause().ok();

    if average_energy > 0.001 {
        println!("[TestVoiceWake] Audio device is responsive. Average energy: {:.6}", average_energy);
        Ok(format!("Audio device OK - Avg energy: {:.6}", average_energy))
    } else {
        println!("[TestVoiceWake] Audio seems quiet or not detecting ambient sound. Average energy: {:.6}", average_energy);
        Ok(format!("Audio device OK - Avg energy: {:.6} (may be quiet)", average_energy))
    }
}

async fn run_wake_loop(
    is_running: Arc<Mutex<bool>>,
    app: AppHandle,
) {
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
    use std::sync::mpsc as std_mpsc;
    
    let (audio_tx, audio_rx) = std_mpsc::channel::<Vec<f32>>();
    
    let host = cpal::default_host();
    let device = match host.default_input_device() {
        Some(d) => {
            println!("[VoiceWake] Using audio device: {:?}", d.name().unwrap_or_else(|_| "Unknown".to_string()));
            d
        },
        None => {
            eprintln!("[VoiceWake] No input device available");
            return;
        }
    };

    let config = match device.default_input_config() {
        Ok(c) => {
            println!("[VoiceWake] Got input config: {:?}", c);
            c
        },
        Err(e) => {
            eprintln!("[VoiceWake] Failed to get input config: {}", e);
            return;
        }
    };

    let sample_rate = config.sample_rate().0;
    let channels = config.channels();

    let audio_tx_clone = audio_tx.clone();
    let stream = match device.build_input_stream(
        &config.into(),
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            let _ = audio_tx_clone.send(data.to_vec());
        },
        |err| eprintln!("[VoiceWake] Stream error: {}", err),
        None,
    ) {
        Ok(s) => {
            println!("[VoiceWake] Successfully built input stream");
            s
        },
        Err(e) => {
            eprintln!("[VoiceWake] Failed to build input stream: {}", e);
            return;
        }
    };

    if let Err(e) = stream.play() {
        eprintln!("[VoiceWake] Failed to play stream: {}", e);
        return;
    }

    let mut state = VoiceState::Idle;
    let threshold = 0.02f32;
    let mut last_activity = std::time::Instant::now();
    let silence_timeout = std::time::Duration::from_millis(3000);
    
    println!("[VoiceWake] Started wake word loop with sample rate: {}, channels: {}, threshold: {}", sample_rate, channels, threshold);

    let vosk_url = std::env::var("VOSK_URL").unwrap_or_else(|_| "ws://192.168.150.26:2700".to_string());
    let mut connection_handle: Option<tokio::sync::oneshot::Sender<()>> = None;
    
    loop {
        let running = *is_running.lock().await;
        if !running {
            println!("[VoiceWake] Stopping wake word loop as requested");
            break;
        }

        std::thread::sleep(std::time::Duration::from_millis(10));

        while let Ok(audio_data) = audio_rx.try_recv() {
            last_activity = std::time::Instant::now();
            let len = audio_data.len();
            
            match state {
                VoiceState::Idle => {
                    let energy: f32 = audio_data.iter().map(|&s| s * s).sum::<f32>() / audio_data.len() as f32;
                    
                    // Periodically log audio information during idle state when detecting possible speech
                    if energy > 0.005 {  // Only log when there's likely human speech activity (>50% of threshold)
                        println!("[VoiceWake][IDLE] Audio detected. Energy: {:.6}, Samples: {}, Threshold: {:.4}", energy, len, threshold);
                        
                        // Additional audio characteristics for debugging
                        let max_val = audio_data.iter().fold(0.0_f32, |acc, &x| acc.max(x.abs()));
                        let avg_abs = audio_data.iter().map(|&x| x.abs()).sum::<f32>() / len as f32;
                        println!("[VoiceWake][IDLE] Max ABS val: {:.6}, Average ABS: {:.6}", max_val, avg_abs);
                        
                        // Show percentage of samples above threshold for insight into continuous activity
                        let above_threshold = audio_data.iter().filter(|&&x| x.abs() > threshold.sqrt()).count();
                        let percent_above = (above_threshold as f32 / len as f32) * 100.0;
                        println!("[VoiceWake][IDLE] {:.1}% of samples above threshold", percent_above);
                    }
                    
                    // Log periodical updates when in idle state for long periods 
                    if last_activity.elapsed().as_millis() % 500 < 10 {  // Every 500ms
                        println!("[VoiceWake][IDLE] Standby - Energy: {:.6}, Max amplitude: {:.6}", energy, audio_data.iter().fold(0.0_f32, |acc, &x| acc.max(x.abs())));
                    }
                    
                     if energy > threshold {
                        state = VoiceState::Waking;
                        println!("[VoiceWake] Wake word detected! Energy: {:.4} > {:.4}", energy, threshold);
                        
                        if let Err(e) = app.emit_all("voice-waked", ()) {
                            eprintln!("[VoiceWake] Failed to emit voice-waked: {}", e);
                        }
                        
                        // Bring the application window to the foreground when woken up
                        // Directly manipulate the window to show, unfocus and bring to front
                        if let Some(window) = app.get_window("main") {
                            if let Err(e) = window.show() {
                                eprintln!("[VoiceWake] Failed to show window: {}", e);
                            }
                            if let Err(e) = window.set_focus() {
                                eprintln!("[VoiceWake] Failed to set focus: {}", e);
                            }
                            if let Err(e) = window.unminimize() {
                                eprintln!("[VoiceWake] Failed to unminimize window: {}", e);
                            }
                            println!("[VoiceWake] Window brought to front after wake detection");
                        } else {
                            eprintln!("[VoiceWake] Main window not found when trying to focus");
                        }
                        
                        println!("[TTS] Playing wake-up response");
                        if let Err(e) = app.emit_all("voice-wake-response", ()) {
                            eprintln!("[VoiceWake] Failed to emit voice-wake-response: {}", e);
                        }
                        
                        state = VoiceState::Listening;
                        if let Err(e) = app.emit_all("voice-state-changed", "listening") {
                            eprintln!("[VoiceWake] Failed to emit voice-state-changed: {}", e);
                        }
                        
                        if connection_handle.is_none() {
                            let app_handle = app.clone();
                            let vosk_uri = vosk_url.clone();
                            let (cancel_tx, cancel_rx) = tokio::sync::oneshot::channel::<()>();
                            connection_handle = Some(cancel_tx);
                            
                            tokio::spawn(async move {
                                run_vosk_client(app_handle, vosk_uri, cancel_rx).await;
                            });
                        }
                    }
                }
                VoiceState::Listening => {
                    if last_activity.elapsed() >= silence_timeout {
                        if let Some(cancel_tx) = connection_handle.take() {
                            if cancel_tx.send(()).is_ok() {
                                println!("[VoiceWake] Cancelled Vosk client connection");
                            }
                        }
                        
                        println!("[VoiceWake] Silence timeout exceeded ({}ms), returning to idle", silence_timeout.as_millis());
                        state = VoiceState::Idle;
                        if let Err(e) = app.emit_all("voice-state-changed", "idle") {
                            eprintln!("[VoiceWake] Failed to emit voice-state-changed to idle: {}", e);
                        }
                    }
                }
                VoiceState::Waking => {
                    state = VoiceState::Listening;
                    if let Err(e) = app.emit_all("voice-state-changed", "listening") {
                        eprintln!("[VoiceWake] Failed to emit voice-state-changed: {}", e);
                    }
                }
                VoiceState::Processing => {
                    // In Processing, treat like Listening until timeout triggers return to Idle
                    if last_activity.elapsed() >= silence_timeout {
                        if let Some(cancel_tx) = connection_handle.take() {
                            if cancel_tx.send(()).is_ok() {
                                println!("[VoiceWake] Cancelled Vosk client connection");
                            }
                        }
                        
                        println!("[VoiceWake] Silence timeout exceeded ({}ms), returning to idle", silence_timeout.as_millis());
                        state = VoiceState::Idle;
                        if let Err(e) = app.emit_all("voice-state-changed", "idle") {
                            eprintln!("[VoiceWake] Failed to emit voice-state-changed to idle: {}", e);
                        }
                    }
                }
            }
        }
    }
    
    if let Some(cancel_tx) = connection_handle.take() {
        let _ = cancel_tx.send(());
    }

    let _ = stream.pause();
    println!("[VoiceWake] Stream paused, wake word loop ended");
}

async fn run_vosk_client(
    app: AppHandle,
    websocket_url: String,
    mut cancel_rx: tokio::sync::oneshot::Receiver<()>
) {
    use tokio_tungstenite::{connect_async, tungstenite::Message};
    use futures_util::SinkExt;
    use futures_util::StreamExt;
    
    match connect_async(&websocket_url).await {
        Ok((mut ws_stream, _)) => {
            let config_msg = serde_json::json!({
                "config": { "sample_rate": 16000 }
            });
            
            if ws_stream.send(Message::Text(config_msg.to_string())).await.is_ok() {
                println!("[VoiceWake] Sent initial Vosk configuration");
                
                loop {
                    tokio::select! {
                        result = ws_stream.next() => {
                            match result {
                                Some(Ok(message)) => {
                                    match message {
                                        Message::Text(text) => {
                                            match serde_json::from_str::<serde_json::Value>(&text) {
                                                Ok(parsed) => {
                                                    if let Some(final_text) = parsed.get("text").and_then(|t| t.as_str()) {
                                                        if !final_text.is_empty() {
                                                            println!("[VoiceWake] ASR Final Text: {}", final_text);
                                                            let _ = app.emit_all("voice-result", serde_json::json!({
                                                                "text": final_text.to_string(),
                                                                "is_final": true
                                                            }));
                                                        }
                                                    } else if let Some(partial_text) = parsed.get("partial").and_then(|p| p.as_str()) {
                                                        if !partial_text.is_empty() {
                                                            println!("[VoiceWake] ASR Partial Text: {}", partial_text);
                                                            let _ = app.emit_all("voice-result", serde_json::json!({
                                                                "text": partial_text.to_string(),
                                                                "is_final": false
                                                            }));
                                                        }
                                                    }
                                                },
                                                Err(e) => {
                                                    eprintln!("[VoiceWake] Could not parse: {} - Error: {}", text, e);
                                                }
                                            }
                                        },
                                        Message::Close(_) => {
                                            println!("[VoiceWake] Vosk connection closed");
                                            break;
                                        },
                                        Message::Ping(_) | Message::Pong(_) | Message::Binary(_) => {
                                            // Do nothing for these message types
                                        }
                                        tokio_tungstenite::tungstenite::Message::Frame(_) => {
                                            // Ignore frame messages
                                        }
                                    }
                                },
                                Some(Err(e)) => {
                                    eprintln!("[VoiceWake] Vosk stream error: {}", e);
                                    break;
                                },
                                None => {
                                    // WebSocket stream closed
                                    println!("[VoiceWake] Vosk connection closed");
                                    break;
                                }
                            }
                        },
                        _ = &mut cancel_rx => {
                            println!("[VoiceWake] Cancel signal received, disconnecting Vosk client");
                            break;
                        }
                    }
                }
            }
        },
        Err(e) => {
            eprintln!("[VoiceWake] Could not connect to Vosk server {}: {}", websocket_url, e);
        }
    }
}