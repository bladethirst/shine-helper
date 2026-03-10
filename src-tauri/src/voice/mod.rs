/// Defines the different voice system states for the wake-up recognition state machine
#[derive(Debug, Clone, PartialEq)]
pub enum VoiceState {
    /// System idle, waiting for wake word
    Idle,
    /// Just after wake word detected, preparing TTS response
    Waking,
    /// Listening for user speech to recognize after wake-up
    Listening,
    /// Processing recognition results
    Processing,
}

pub mod audio_capture;
pub mod wake_word;
pub mod asr_client;
pub mod tts_player;
pub mod state_machine;
pub mod recognition;

pub use audio_capture::*;
pub use wake_word::*;
pub use asr_client::*;
pub use tts_player::*;
pub use state_machine::*;
pub use recognition::*;
