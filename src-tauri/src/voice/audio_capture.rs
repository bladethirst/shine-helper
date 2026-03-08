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
        sample_rate: u32,
        channels: u16,
        sender: mpsc::Sender<Vec<f32>>,
    ) -> Result<(), String> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or("No input device available")?;

        let config = device
            .default_input_config()
            .map_err(|e| e.to_string())?;

        self.is_running.store(true, Ordering::SeqCst);

        let is_running = Arc::clone(&self.is_running);

        let err_fn = |err| eprintln!("[AudioCapture] Error: {}", err);

        let stream = device
            .build_input_stream(
                &config.into(),
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    if is_running.load(Ordering::SeqCst) {
                        let _ = sender.try_send(data.to_vec());
                    }
                },
                err_fn,
                None,
            )
            .map_err(|e| e.to_string())?;

        stream
            .play()
            .map_err(|e| e.to_string())?;

        self.stream = Some(stream);
        Ok(())
    }

    pub fn stop(&mut self) {
        self.is_running.store(false, Ordering::SeqCst);
        self.stream = None;
    }
}

pub fn resample_to_16k_mono(data: &[f32], from_sample_rate: u32, from_channels: u16) -> Vec<i16> {
    let mut result = Vec::new();
    let ratio = from_sample_rate as f32 / 16000.0;
    let mut i = 0;
    
    while (i as f32 * ratio) as usize < data.len() {
        let sample = data[(i as f32 * ratio) as usize];
        let sample_i16 = (sample * 32767.0) as i16;
        result.push(sample_i16);
        i += 1;
    }
    
    result
}