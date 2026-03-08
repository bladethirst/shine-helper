use crate::config::VoiceConfig;
use super::{AudioCapture, WakeWordDetector, QwenAsrClient, TtsPlayer};

#[derive(Debug, Clone, PartialEq)]
pub enum VoiceState {
    Idle,
    Waking,
    Listening,
    Processing,
}

pub struct VoiceStateMachine {
    pub state: VoiceState,
    pub audio_capture: AudioCapture,
    pub wake_detector: WakeWordDetector,
    pub asr_client: QwenAsrClient,
    pub tts_player: TtsPlayer,
    pub config: VoiceConfig,
}

impl VoiceStateMachine {
    pub fn new(config: VoiceConfig) -> Result<Self, String> {
        Ok(Self {
            state: VoiceState::Idle,
            audio_capture: AudioCapture::new()?,
            wake_detector: WakeWordDetector::new(&config.wake_word),
            asr_client: QwenAsrClient::new(&config.qwen_asr_url, &config.qwen_asr_api_key),
            tts_player: TtsPlayer::new(config.wake_sounds.clone()),
            config,
        })
    }

    pub fn transition_to(&mut self, new_state: VoiceState) {
        println!("[Voice] State: {:?} -> {:?}", self.state, new_state);
        self.state = new_state;
    }
}