use reqwest::Client;
use serde_json::json;
use std::error::Error;

pub struct GeminiClient {
    api_key: String,
    client: Client,
}

impl GeminiClient {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: Client::new(),
        }
    }

    pub async fn generate_content(&self, task: &str, system_prompt: &str) -> Result<String, Box<dyn Error>> {
        let response = self.client
            .post("https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent")
            .query(&[("key", &self.api_key)])
            .json(&json!({
                "system_instruction": {
                    "parts": [{
                        "text": system_prompt
                    }]
                },
                "contents": [{
                    "parts": [{
                        "text": task
                    }]
                }]
            }))
            .send()
            .await?;

        let result = response.json::<serde_json::Value>().await?;
        
        let text = result
            .get("candidates")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("content"))
            .and_then(|c| c.get("parts"))
            .and_then(|p| p.get(0))
            .and_then(|p| p.get("text"))
            .and_then(|t| t.as_str())
            .ok_or_else(|| {
                let error = format!("Unexpected response format: {}", result);
                std::io::Error::new(std::io::ErrorKind::Other, error)
            })?;

        Ok(text.to_string())
    }
} 