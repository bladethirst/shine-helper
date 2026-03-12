use rodio::{Decoder, Source};
use std::io::{BufReader, Cursor};
use std::path::PathBuf;
use rand::prelude::SliceRandom;

#[derive(Clone)]
pub struct TtsPlayer {
    wake_sounds: Vec<String>,
    resource_dir: Option<PathBuf>,
}

impl TtsPlayer {
    pub fn new(wake_sounds: Vec<String>) -> Self {
        Self {
            wake_sounds,
            resource_dir: None,
        }
    }

    pub fn with_resource_dir(mut self, resource_dir: PathBuf) -> Self {
        self.resource_dir = Some(resource_dir);
        self
    }

    pub fn play_wake_response(&self) -> Result<(), String> {
        use rand::Rng;
        
        let response = self
            .wake_sounds
            .choose(&mut rand::thread_rng())
            .cloned()
            .unwrap_or_else(|| "在呢".to_string());

        println!("[TTS] Playing response: {}", response);
        
        if let Some(ref resource_dir) = self.resource_dir {
            let mp3_path = resource_dir.join(format!("{}.mp3", response));
            if mp3_path.exists() {
                println!("[TTS] Playing from file: {:?}", mp3_path);
                return self.play_audio_file(mp3_path.to_str().unwrap());
            } else {
                println!("[TTS] Audio file not found: {:?}, using beep sound", mp3_path);
            }
        }
        
        self.play_beep_sound()
    }

    fn play_beep_sound(&self) -> Result<(), String> {
        use rodio::{OutputStream, Sink};
        use std::time::Duration;
        
        let (_stream, stream_handle) = OutputStream::try_default()
            .map_err(|e| format!("Failed to create audio stream: {}", e))?;
        
        let sink = Sink::try_new(&stream_handle)
            .map_err(|e| format!("Failed to create sink: {}", e))?;
        
        let duration = Duration::from_millis(500);
        let sample_rate = 44100;
        let frequency = 440.0;
        
        let samples: Vec<f32> = (0..(sample_rate * duration.as_secs() as u32 + sample_rate / 2))
            .map(|t| {
                let time = t as f32 / sample_rate as f32;
                (frequency * time * 2.0 * std::f32::consts::PI).sin() * 0.3
            })
            .collect();
        
        let pcm_data: Vec<u8> = samples
            .into_iter()
            .flat_map(|s| {
                let sample = (s * 32767.0) as i16;
                vec![(sample & 0xFF) as u8, (sample >> 8) as u8]
            })
            .collect();
        
        let wav_data = Self::create_wav_file(&pcm_data, sample_rate as u32, 1, 16);
        
        let cursor = Cursor::new(wav_data);
        let source = Decoder::new(BufReader::new(cursor))
            .map_err(|e| format!("Failed to decode audio: {}", e))?;
        
        sink.append(source);
        
        // 等待播放完成 - 使用 sleep_until_end
        sink.sleep_until_end();
        
        println!("[TTS] Beep sound played");
        
        Ok(())
    }

    fn create_wav_file(pcm_data: &[u8], sample_rate: u32, channels: u16, bits_per_sample: u16) -> Vec<u8> {
        let subchunk2_size = pcm_data.len() as u32;
        let chunk_size = 36 + subchunk2_size;
        let byte_rate = sample_rate * channels as u32 * (bits_per_sample / 8) as u32;
        let block_align = channels * (bits_per_sample / 8);
        
        let mut wav = Vec::new();
        wav.extend_from_slice(b"RIFF");
        wav.extend_from_slice(&chunk_size.to_le_bytes());
        wav.extend_from_slice(b"WAVE");
        wav.extend_from_slice(b"fmt ");
        wav.extend_from_slice(&16u32.to_le_bytes());
        wav.extend_from_slice(&1u16.to_le_bytes());
        wav.extend_from_slice(&channels.to_le_bytes());
        wav.extend_from_slice(&sample_rate.to_le_bytes());
        wav.extend_from_slice(&byte_rate.to_le_bytes());
        wav.extend_from_slice(&block_align.to_le_bytes());
        wav.extend_from_slice(&bits_per_sample.to_le_bytes());
        wav.extend_from_slice(b"data");
        wav.extend_from_slice(&subchunk2_size.to_le_bytes());
        wav.extend_from_slice(pcm_data);
        wav
    }

    pub fn play_audio_file(&self, file_path: &str) -> Result<(), String> {
        use rodio::{OutputStream, Sink};
        use std::fs::File;
        
        let (_stream, stream_handle) = OutputStream::try_default()
            .map_err(|e| format!("Failed to create audio stream: {}", e))?;
        
        let sink = Sink::try_new(&stream_handle)
            .map_err(|e| format!("Failed to create sink: {}", e))?;
        
        let file = File::open(file_path)
            .map_err(|e| format!("Failed to open audio file: {}", e))?;
        
        let source = Decoder::new(BufReader::new(file))
            .map_err(|e| format!("Failed to decode audio: {}", e))?;
        
        sink.append(source);
        
        // 等待播放完成
        sink.sleep_until_end();
        
        println!("[TTS] Audio file played: {}", file_path);
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tts_player_creation() {
        let player = TtsPlayer::new(vec!["在呢".to_string()]);
        assert_eq!(player.wake_sounds.len(), 1);
    }

    #[test]
    fn test_create_wav_file() {
        let pcm_data = vec![0u8; 1024];
        let wav = TtsPlayer::create_wav_file(&pcm_data, 44100, 1, 16);
        
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");
        assert_eq!(&wav[12..16], b"fmt ");
        assert_eq!(&wav[36..40], b"data");
    }
}
