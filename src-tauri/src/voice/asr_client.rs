use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use url::Url;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsrResult {
    pub text: String,
    pub is_final: bool,
    pub confidence: f32,
}

pub struct QwenAsrClient {
    url: String,
    api_key: String,
}

impl QwenAsrClient {
    pub fn new(url: &str, api_key: &str) -> Self {
        Self {
            url: url.to_string(),
            api_key: api_key.to_string(),
        }
    }

    pub async fn connect(
        &self,
        result_sender: mpsc::Sender<AsrResult>,
    ) -> Result<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>, String> {
        let ws_url = if self.api_key.is_empty() {
            self.url.clone()
        } else {
            format!("{}?api_key={}", self.url, self.api_key)
        };

        let url = Url::parse(&ws_url).map_err(|e| e.to_string())?;
        let (ws_stream, _) = connect_async(url)
            .await
            .map_err(|e| e.to_string())?;

        let (mut write, mut read) = ws_stream.split();

        let config = serde_json::json!({
            "type": "config",
            "format": "pcm_16000",
            "sample_rate": 16000,
            "channels": 1
        });
        write
            .send(Message::Text(config.to_string()))
            .await
            .map_err(|e| e.to_string())?;

        tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        if let Ok(result) = serde_json::from_str::<AsrResult>(&text) {
                            let _ = result_sender.send(result).await;
                        }
                    }
                    Ok(Message::Close(_)) => break,
                    Err(e) => {
                        eprintln!("[QwenASR] WebSocket error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
        });

        Ok(ws_stream)
    }

    pub async fn send_audio(&self, ws: &mut tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>, audio_data: Vec<i16>) -> Result<(), String> {
        let bytes: Vec<u8> = audio_data
            .iter()
            .flat_map(|&s| s.to_le_bytes())
            .collect();

        ws.send(Message::Binary(bytes))
            .await
            .map_err(|e| e.to_string())
    }
}