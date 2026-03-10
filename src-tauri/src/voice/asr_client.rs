use futures_util::{SinkExt, StreamExt};
use serde::{Serialize, Deserialize};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsrResult {
    pub text: String,
    pub is_final: bool,
}

pub struct VoskAsrClient {
    url: String,
    api_key: Option<String>,
}

impl VoskAsrClient {
    pub fn new(url: &str, api_key: Option<&str>) -> Self {
        Self {
            url: url.to_string(),
            api_key: api_key.map(|s| s.to_string()),
        }
    }

    pub async fn connect(
        &self,
    ) -> Result<(mpsc::Sender<Vec<i16>>, mpsc::Receiver<AsrResult>), String> {
        let url = if let Some(ref api_key) = self.api_key {
            format!("{}?api_key={}", self.url, api_key)
        } else {
            self.url.clone()
        };

        let (ws_stream, _) = connect_async(&url)
            .await
            .map_err(|e| format!("Failed to connect to Vosk: {}", e))?;

        let (mut write, mut read) = ws_stream.split();

        // 发送配置
        let config = serde_json::json!({
            "config": { "sample_rate": 16000 }
        });
        write
            .send(Message::Text(config.to_string()))
            .await
            .map_err(|e| e.to_string())?;

        // 创建通道
        let (audio_tx, mut audio_rx) = mpsc::channel::<Vec<i16>>(100);
        let (result_tx, result_rx) = mpsc::channel::<AsrResult>(100);

        // 启动音频发送任务
        tokio::spawn(async move {
            while let Some(audio_data) = audio_rx.recv().await {
                // 转换为字节数组
                let bytes: Vec<u8> = audio_data
                    .iter()
                    .flat_map(|&s| s.to_le_bytes())
                    .collect();

                if write.send(Message::Binary(bytes)).await.is_err() {
                    break;
                }
            }
        });

        // 启动结果接收任务
        tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        if let Ok(result) = serde_json::from_str::<serde_json::Value>(&text) {
                            // 处理 Vosk 返回结果
                            if let Some(partial) = result.get("partial").and_then(|p| p.as_str()) {
                                if !partial.is_empty() {
                                    let _ = result_tx.send(AsrResult {
                                        text: partial.to_string(),
                                        is_final: false,
                                    }).await;
                                }
                            } else if let Some(result_text) = result.get("text").and_then(|t| t.as_str()) {
                                if !result_text.is_empty() {
                                    let _ = result_tx.send(AsrResult {
                                        text: result_text.to_string(),
                                        is_final: true,
                                    }).await;
                                }
                            }
                        }
                    }
                    Ok(Message::Close(_)) => break,
                    Err(e) => {
                        eprintln!("[VoskASR] WebSocket error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
        });

        Ok((audio_tx, result_rx))
    }
}
