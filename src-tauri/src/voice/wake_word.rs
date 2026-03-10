use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// 简化的唤醒词检测（基于能量阈值）
/// 注意：生产环境应使用 Porcupine/Snowboy 等专业引擎
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
            threshold: 0.02, // 能量阈值
        }
    }

    /// 检测音频数据是否包含唤醒词
    /// 简化版本：返回是否检测到语音活动
    pub fn detect(&self, audio_data: &[f32]) -> bool {
        if !self.is_enabled.load(Ordering::SeqCst) {
            return false;
        }

        // 计算音频能量
        let energy: f32 = audio_data.iter().map(|&s| s * s).sum::<f32>() / audio_data.len() as f32;
        
        // 简单 VAD (Voice Activity Detection)
        energy > self.threshold
    }

    pub fn enable(&mut self) {
        self.is_enabled.store(true, Ordering::SeqCst);
    }

    pub fn disable(&mut self) {
        self.is_enabled.store(false, Ordering::SeqCst);
    }
}

/// 检测是否包含结束词
pub fn contains_end_word(text: &str, end_words: &[String]) -> bool {
    end_words.iter().any(|word| text.contains(word))
}
