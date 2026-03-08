use std::sync::Arc;
use std::sync::Mutex;

pub struct TtsPlayer {
    wake_sounds: Vec<String>,
}

impl TtsPlayer {
    pub fn new(wake_sounds: Vec<String>) -> Self {
        Self {
            wake_sounds,
        }
    }

    /// Play random wake response
    /// Simplified version: logs the response
    /// Production should integrate TTS service or play preset audio files
    pub fn play_wake_response(&self) -> Result<String, String> {
        let response = self.wake_sounds
            .get(rand::random::<usize>() % self.wake_sounds.len())
            .cloned()
            .unwrap_or_else(|| "在呢".to_string());
        
        println!("[TTS] 播放回复：{}", response);
        Ok(response)
    }

    pub fn stop(&self) {
        // No-op for simplified version
    }
}