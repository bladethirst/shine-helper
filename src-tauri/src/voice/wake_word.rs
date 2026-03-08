use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Simplified wake word detection (based on energy threshold)
/// Note: Production should use Porcupine/Snowboy professional engine
pub struct WakeWordDetector {
    wake_word: String,
    is_enabled: Arc<AtomicBool>,
    threshold: f32,
}

impl WakeWordDetector {
    pub fn new(wake_word: &str) -> Self {
        Self {
            wake_word: wake_word.to_string(),
            is_enabled: Arc::new(AtomicBool::new(true)),
            threshold: 0.02,
        }
    }

    /// Detect if audio contains wake word
    /// Simplified version: returns whether voice activity detected
    pub fn detect(&self, audio_data: &[f32]) -> bool {
        if !self.is_enabled.load(Ordering::SeqCst) {
            return false;
        }

        let energy: f32 = audio_data.iter().map(|&s| s * s).sum::<f32>() / audio_data.len() as f32;
        
        energy > self.threshold
    }

    pub fn enable(&mut self) {
        self.is_enabled.store(true, Ordering::SeqCst);
    }

    pub fn disable(&mut self) {
        self.is_enabled.store(false, Ordering::SeqCst);
    }
}

/// Check if text contains end word
pub fn contains_end_word(text: &str, end_words: &[String]) -> bool {
    end_words.iter().any(|word| text.contains(word))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wake_word_detector_new() {
        let detector = WakeWordDetector::new("hey assistant");
        assert_eq!(detector.wake_word, "hey assistant");
        assert!(detector.is_enabled.load(Ordering::SeqCst));
        assert_eq!(detector.threshold, 0.02);
    }

    #[test]
    fn test_detect_with_silent_audio() {
        let detector = WakeWordDetector::new("hey assistant");
        let silent_audio = vec![0.0f32; 1000];
        assert!(!detector.detect(&silent_audio));
    }

    #[test]
    fn test_detect_with_loud_audio() {
        let detector = WakeWordDetector::new("hey assistant");
        let loud_audio = vec![0.5f32; 1000];
        assert!(detector.detect(&loud_audio));
    }

    #[test]
    fn test_enable_disable() {
        let mut detector = WakeWordDetector::new("hey assistant");
        assert!(detector.is_enabled.load(Ordering::SeqCst));
        
        detector.disable();
        assert!(!detector.is_enabled.load(Ordering::SeqCst));
        
        let loud_audio = vec![0.5f32; 1000];
        assert!(!detector.detect(&loud_audio));
        
        detector.enable();
        assert!(detector.is_enabled.load(Ordering::SeqCst));
        assert!(detector.detect(&loud_audio));
    }

    #[test]
    fn test_contains_end_word() {
        let end_words = vec!["stop".to_string(), "end".to_string(), "quit".to_string()];
        
        assert!(contains_end_word("please stop now", &end_words));
        assert!(contains_end_word("I want to end", &end_words));
        assert!(contains_end_word("quit the app", &end_words));
        assert!(!contains_end_word("continue", &end_words));
    }
}