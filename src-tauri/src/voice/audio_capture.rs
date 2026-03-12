use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;

pub struct AudioCapture {
    stream: Option<Stream>,
    is_running: Arc<AtomicBool>,
}

impl AudioCapture {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            stream: None,
            is_running: Arc::new(AtomicBool::new(false)),
        })
    }

    pub fn start(
        &mut self,
        _sample_rate: u32,
        _channels: u16,
        sender: mpsc::UnboundedSender<Vec<f32>>,
    ) -> Result<(), String> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or("No input device available")?;

        let config = device
            .default_input_config()
            .map_err(|e| e.to_string())?;

        println!("[AudioCapture] Using device: {}", device.name().unwrap_or_else(|_| "Unknown".to_string()));
        println!("[AudioCapture] Sample rate: {}, Channels: {}", config.sample_rate().0, config.channels());

        self.is_running.store(true, Ordering::SeqCst);

        let is_running = Arc::clone(&self.is_running);
        let sender_clone = sender.clone();

        let err_fn = |err| eprintln!("[AudioCapture] Error: {}", err);

        let mut frame_count = 0u32;
        let stream = device
            .build_input_stream(
                &config.into(),
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    frame_count += 1;
                    // if frame_count % 100 == 0 {
                    //     println!("[AudioCapture] Captured {} frames, data.len={}", frame_count, data.len());
                    // }
                    if is_running.load(Ordering::SeqCst) {
                        let _ = sender_clone.send(data.to_vec());
                    }
                },
                err_fn,
                None,
            )
            .map_err(|e| e.to_string())?;

        stream
            .play()
            .map_err(|e| e.to_string())?;

        println!("[AudioCapture] Stream started");

        self.stream = Some(stream);
        Ok(())
    }

    pub fn stop(&mut self) {
        self.is_running.store(false, Ordering::SeqCst);
        self.stream = None;
    }
}

/// 将音频数据转换为 16kHz 单声道 PCM
pub fn resample_to_16k_mono(data: &[f32], from_sample_rate: u32, from_channels: u16) -> Vec<i16> {
    let mut result = Vec::new();
    
    // 简单下采样到 16kHz
    let ratio = from_sample_rate as f32 / 16000.0;
    let mut i = 0;
    
    while ((i as f32 * ratio) as usize) < data.len() {
        // 如果是多声道，取第一个声道
        let sample = data[(i as f32 * ratio) as usize];
        // 转换为 16-bit PCM
        let sample_i16 = (sample * 32767.0) as i16;
        result.push(sample_i16);
        i += 1;
    }
    
    result
}
