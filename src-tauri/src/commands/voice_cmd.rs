use crate::voice::{VoiceStateMachine, VoiceState};
use crate::config::VoiceConfig;
use std::sync::Arc;
use tauri::{State, Manager, AppHandle};
use tokio::sync::Mutex;

pub struct VoiceAppState {
    pub state_machine: Arc<Mutex<Option<VoiceStateMachine>>>,
    pub config: Arc<Mutex<VoiceConfig>>,
}

#[tauri::command]
pub async fn start_voice_wake(
    state: State<'_, VoiceAppState>,
    app: AppHandle,
) -> Result<(), String> {
    let config = state.config.lock().await.clone();
    
    let mut sm_guard = state.state_machine.lock().await;
    *sm_guard = Some(VoiceStateMachine::new(config.clone())?);
    
    // TODO: Start background listening task
    
    Ok(())
}

#[tauri::command]
pub async fn stop_voice_wake(state: State<'_, VoiceAppState>) -> Result<(), String> {
    let mut sm_guard = state.state_machine.lock().await;
    if let Some(sm) = sm_guard.as_mut() {
        sm.audio_capture.stop();
        sm.tts_player.stop();
        sm.transition_to(VoiceState::Idle);
    }
    Ok(())
}

#[tauri::command]
pub async fn set_voice_config(
    state: State<'_, VoiceAppState>,
    config: VoiceConfig,
) -> Result<(), String> {
    let mut config_guard = state.config.lock().await;
    *config_guard = config;
    Ok(())
}

#[tauri::command]
pub async fn get_voice_config(
    state: State<'_, VoiceAppState>,
) -> Result<VoiceConfig, String> {
    let config = state.config.lock().await.clone();
    Ok(config)
}