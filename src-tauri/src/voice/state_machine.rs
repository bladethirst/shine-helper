use crate::voice::{AudioCapture, WakeWordDetector, VoskAsrClient, TtsPlayer, AsrResult};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct VoiceStateMachine {
    pub state: super::VoiceState,
    pub audio_capture: AudioCapture,
    pub wake_detector: WakeWordDetector,
    pub tts_player: TtsPlayer,
    pub wake_word: String,
    pub wake_sounds: Vec<String>,
    pub end_words: Vec<String>,
    pub silence_timeout: u32,
}

impl VoiceStateMachine {
    pub fn new(
        wake_word: String,
        wake_sounds: Vec<String>,
        end_words: Vec<String>,
        silence_timeout: u32,
    ) -> Result<Self, String> {
        Ok(Self {
            state: super::VoiceState::Idle,
            audio_capture: AudioCapture::new()?,
            wake_detector: WakeWordDetector::new(&wake_word),
            tts_player: TtsPlayer::new(wake_sounds.clone()),
            wake_word,
            wake_sounds,
            end_words,
            silence_timeout,
        })
    }

    pub fn transition_to(&mut self, new_state: super::VoiceState) {
        println!("[Voice] State: {:?} -> {:?}", self.state, new_state);
        self.state = new_state;
    }
}
