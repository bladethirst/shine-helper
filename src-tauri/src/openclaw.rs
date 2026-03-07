use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use bytes::BytesMut;

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    #[serde(rename = "input")]
    input: String,
}

#[derive(Debug, Serialize)]
struct ChatRequestStream {
    model: String,
    #[serde(rename = "input")]
    input: String,
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct OpenClawOutputItem {
    #[serde(rename = "type")]
    output_type: String,
    #[serde(rename = "id")]
    id: Option<String>,
    #[serde(rename = "content")]
    content: Option<Vec<OpenClawContentBlock>>,
    #[serde(rename = "message")]
    message: Option<OpenClawMessage>,
}

#[derive(Debug, Deserialize)]
struct OpenClawContentBlock {
    #[serde(rename = "type")]
    content_type: String,
    #[serde(rename = "text")]
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenClawMessage {
    #[serde(rename = "content")]
    content: Option<String>,
    #[serde(rename = "role")]
    role: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenClawResponse {
    id: String,
    object: String,
    status: String,
    model: String,
    output: Vec<OpenClawOutputItem>,
}

#[derive(Debug, Deserialize)]
struct SSEvent {
    #[serde(rename = "type")]
    event_type: String,
    delta: Option<String>,
}

pub struct OpenClawClient {
    client: Client,
    base_url: String,
    token: String,
}

impl OpenClawClient {
    pub fn new(base_url: &str) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .unwrap_or_default();
            
        Self {
            client,
            base_url: base_url.to_string(),
            token: String::new(),
        }
    }

    pub fn set_token(&mut self, token: String) {
        self.token = token;
    }

    pub async fn chat(&self, message: &str) -> Result<String, String> {
        let url = format!("{}/v1/responses", self.base_url);
        
        let request = self.client.post(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Content-Type", "application/json")
            .json(&ChatRequest {
                model: "openclaw".to_string(),
                input: message.to_string(),
            });
        
        let response = request.send().await
            .map_err(|e| format!("request failed: {}", e))?;
            
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("API error ({}): {}", status, text));
        }
        
        let text = response.text().await
            .map_err(|e| format!("read failed: {}", e))?;
        
        let resp: OpenClawResponse = serde_json::from_str(&text)
            .map_err(|e| format!("parse error: {} - response: {}", e, &text[..text.len().min(200)]))?;
        
        let mut result = String::new();
        for item in resp.output {
            if item.output_type == "message" {
                if let Some(message) = item.message {
                    if let Some(text) = message.content {
                        result.push_str(&text);
                    }
                }
                if let Some(blocks) = item.content {
                    for block in blocks {
                        if block.content_type == "output_text" {
                            if let Some(text) = block.text {
                                result.push_str(&text);
                            }
                        }
                    }
                }
            }
        }
        
        Ok(result)
    }

    pub async fn chat_stream<F>(&self, message: &str, mut on_chunk: F) -> Result<(), String>
    where
        F: FnMut(String) + Send,
    {
        let url = format!("{}/v1/responses", self.base_url);
        
        let request = self.client.post(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Content-Type", "application/json")
            .json(&ChatRequestStream {
                model: "openclaw".to_string(),
                input: message.to_string(),
                stream: true,
            });
        
        let response = request.send().await
            .map_err(|e| format!("request failed: {}", e))?;
            
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("API error ({}): {}", status, text));
        }
        
        use futures_util::stream::StreamExt;
        
        let mut stream = response.bytes_stream();
        let mut buffer = BytesMut::new();
        
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.map_err(|e| format!("read chunk failed: {}", e))?;
            buffer.extend_from_slice(&chunk);
            
            while let Some(pos) = buffer.windows(1).position(|w| w[0] == b'\n') {
                let line = String::from_utf8_lossy(&buffer[..pos]).to_string();
                buffer = buffer[pos + 1..].into();
                
                if line.starts_with("data: ") {
                    let data = line[6..].trim();
                    if data.is_empty() || data == "[DONE]" {
                        continue;
                    }
                    
                    if let Ok(event) = serde_json::from_str::<serde_json::Value>(data) {
                        if let Some(event_type) = event.get("type").and_then(|v| v.as_str()) {
                            if event_type == "response.output_text.delta" {
                                if let Some(delta) = event.get("delta").and_then(|v| v.as_str()) {
                                    on_chunk(delta.to_string());
                                }
                            } else if event_type == "response.completed" || event_type == "response.output_text.done" {
                                return Ok(());
                            }
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
}