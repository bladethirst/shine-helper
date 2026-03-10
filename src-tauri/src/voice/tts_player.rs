use std::sync::Arc;
use std::sync::Mutex;

pub struct TtsPlayer {
    wake_sounds: Vec<String>,
}

impl TtsPlayer {
    pub fn new(wake_sounds: Vec<String>) -> Self {
        Self { wake_sounds }
    }

    /// 播放随机唤醒回复
    pub fn play_wake_response(&self) -> Result<(), String> {
        // 简化版本：打印日志，实际应播放音频
        // 生产环境应集成 TTS 服务或预置音频文件
        let response = self
            .wake_sounds
            .first()
            .cloned()
            .unwrap_or_else(|| "在呢".to_string());

        println!("[TTS] 播放回复：{}", response);
        Ok(())
    }
}
