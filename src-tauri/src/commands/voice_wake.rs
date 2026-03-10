use crate::voice::VoiceState;
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
    
    // 在新线程中运行音频捕获
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

async fn run_wake_loop(
    is_running: Arc<Mutex<bool>>,
    app: AppHandle,
) {
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
    use std::sync::mpsc as std_mpsc;
    
    let (audio_tx, audio_rx) = std_mpsc::channel::<Vec<f32>>();
    
    let host = cpal::default_host();
    let device = match host.default_input_device() {
        Some(d) => d,
        None => {
            eprintln!("[VoiceWake] No input device available");
            return;
        }
    };

    let config = match device.default_input_config() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[VoiceWake] Failed to get input config: {}", e);
            return;
        }
    };

    let audio_tx_clone = audio_tx.clone();
    let stream = match device.build_input_stream(
        &config.into(),
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            let _ = audio_tx_clone.send(data.to_vec());
        },
        |err| eprintln!("[VoiceWake] Stream error: {}", err),
        None,
    ) {
        Ok(s) => s,
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

    loop {
        let running = *is_running.lock().await;
        if !running {
            break;
        }

        std::thread::sleep(std::time::Duration::from_millis(10));

        while let Ok(audio_data) = audio_rx.try_recv() {
            last_activity = std::time::Instant::now();
            
            match state {
                VoiceState::Idle => {
                    let energy: f32 = audio_data.iter().map(|&s| s * s).sum::<f32>() / audio_data.len() as f32;
                    
                    if energy > threshold {
                        state = VoiceState::Waking;
                        println!("[VoiceWake] Wake word detected!");
                        let _ = app.emit_all("voice-waked", ());
                        println!("[TTS] 播放回复：在呢");
                        state = VoiceState::Listening;
                    }
                }
                VoiceState::Listening => {
                    if last_activity.elapsed() >= silence_timeout {
                        println!("[VoiceWake] Silence timeout, returning to idle");
                        state = VoiceState::Idle;
                        let _ = app.emit_all("voice-state-changed", "idle");
                    }
                }
                _ => {}
            }
        }
    }

    let _ = stream.pause();
}
