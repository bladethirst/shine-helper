use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Serialize)]
struct ChatRequest {
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    history: Option<Vec<MessageItem>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MessageItem {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    response: String,
}

pub struct OpenClawClient {
    client: Client,
    base_url: String,
    api_key: Option<String>,
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
            api_key: None,
        }
    }

    pub fn set_api_key(&mut self, key: String) {
        self.api_key = Some(key);
    }

    pub async fn chat(&self, message: &str) -> Result<String, String> {
        let url = format!("{}/chat", self.base_url);
        
        let mut request = self.client.post(&url).json(&ChatRequest {
            message: message.to_string(),
            history: None,
        });
        
        if let Some(ref key) = self.api_key {
            request = request.header("Authorization", format!("Bearer {}", key));
        }
        
        let response = request.send().await
            .map_err(|e| e.to_string())?;
            
        let chat_resp: ChatResponse = response.json().await
            .map_err(|e| e.to_string())?;
            
        Ok(chat_resp.response)
    }
}
