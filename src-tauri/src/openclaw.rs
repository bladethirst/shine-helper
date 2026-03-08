use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use bytes::BytesMut;

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    #[serde(rename = "input")]
    input: String,
    #[serde(rename = "user")]
    #[serde(skip_serializing_if = "Option::is_none")]
    user: Option<String>,
}

#[derive(Debug, Serialize)]
struct ChatRequestStream {
    model: String,
    #[serde(rename = "input")]
    input: String,
    stream: bool,
    #[serde(rename = "user")]
    #[serde(skip_serializing_if = "Option::is_none")]
    user: Option<String>,
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
    #[serde(rename = "sessionKey")]
    session_key: Option<String>,
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
    session_key: Option<String>,
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
            session_key: None,
        }
    }

    pub fn set_token(&mut self, token: String) {
        self.token = token;
    }

    pub fn set_session_key(&mut self, session_key: Option<String>) {
        self.session_key = session_key;
    }

    pub fn get_session_key(&self) -> Option<&String> {
        self.session_key.as_ref()
    }

    pub async fn chat(&mut self, message: &str, user_id: Option<&str>) -> Result<(String, Option<String>), String> {
        let url = format!("{}/v1/responses", self.base_url);
        
        let request = self.client.post(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Content-Type", "application/json")
            .header("x-openclaw-agent-id", "main")
            .json(&ChatRequest {
                model: "openclaw".to_string(),
                input: message.to_string(),
                user: user_id.map(|s| s.to_string()),
            });
        
        let response = request.send().await
            .map_err(|e| format!("request failed: {}", e))?;
            
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("API error ({}): {}", status, text));
        }
        
        let header_session_key = response.headers()
            .get("x-openclaw-session-key")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());
        
        let text = response.text().await
            .map_err(|e| format!("read failed: {}", e))?;
        
        let resp: OpenClawResponse = serde_json::from_str(&text)
            .map_err(|e| format!("parse error: {} - response: {}", e, &text[..text.len().min(200)]))?;
        
        let mut new_session_key = resp.session_key.or(header_session_key);
        
        if let Some(ref session_key) = new_session_key {
            self.session_key = Some(session_key.clone());
        }
        
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
        
        Ok((result, self.session_key.clone()))
    }

    pub async fn chat_stream<F>(&mut self, message: &str, user_id: Option<&str>, mut on_chunk: F) -> Result<Option<String>, String>
    where
        F: FnMut(String) + Send,
    {
        let url = format!("{}/v1/responses", self.base_url);
        
        let request = self.client.post(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Content-Type", "application/json")
            .header("x-openclaw-agent-id", "main")
            .json(&ChatRequestStream {
                model: "openclaw".to_string(),
                input: message.to_string(),
                stream: true,
                user: user_id.map(|s| s.to_string()),
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
        let mut final_session_key: Option<String> = None;
        
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
                                self.session_key = final_session_key.clone();
                                return Ok(final_session_key);
                            }
                        }
                        
                        if let Some(event_obj) = event.get("event").and_then(|v| v.as_str()) {
                            if event_obj == "agent" {
                                if let Some(payload) = event.get("payload") {
                                    if let Some(session_key) = payload.get("sessionKey").and_then(|v| v.as_str()) {
                                        final_session_key = Some(session_key.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Ok(final_session_key)
    }
}