//! Claude API client
//!
//! Provides async interface to Anthropic's Claude API for generating
//! game optimization recommendations.

use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::sync::mpsc;

use crate::config::Config;

const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";
const DEFAULT_MODEL: &str = "claude-sonnet-4-20250514";
const ANTHROPIC_VERSION: &str = "2023-06-01";

/// Claude API client
pub struct ClaudeClient {
    client: Client,
    api_key: String,
    model: String,
}

/// Error types for Claude API operations
#[derive(Debug, thiserror::Error)]
pub enum ClaudeError {
    #[error("API key not configured. Set claude_api_key in config or ANTHROPIC_API_KEY env var")]
    MissingApiKey,

    #[error("API request failed: {0}")]
    RequestFailed(String),

    #[error("Failed to parse response: {0}")]
    ParseError(String),

    #[error("Rate limited. Please wait and try again")]
    RateLimited,

    #[error("API error: {0}")]
    ApiError(String),
}

impl ClaudeClient {
    /// Create a new Claude client with API key from config or environment
    pub fn new() -> Result<Self, ClaudeError> {
        let api_key = Self::get_api_key()?;
        
        let client = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .map_err(|e| ClaudeError::RequestFailed(e.to_string()))?;

        Ok(Self {
            client,
            api_key,
            model: DEFAULT_MODEL.to_string(),
        })
    }

    /// Create with a specific model
    pub fn with_model(model: &str) -> Result<Self, ClaudeError> {
        let mut client = Self::new()?;
        client.model = model.to_string();
        Ok(client)
    }

    /// Get API key from config or environment
    fn get_api_key() -> Result<String, ClaudeError> {
        // Try environment variable first
        if let Ok(key) = std::env::var("ANTHROPIC_API_KEY") {
            if !key.is_empty() {
                return Ok(key);
            }
        }

        // Try config file
        if let Ok(config) = Config::load() {
            if let Some(key) = config.claude_api_key {
                if !key.is_empty() {
                    return Ok(key);
                }
            }
        }

        Err(ClaudeError::MissingApiKey)
    }

    /// Send a message to Claude and get a response
    pub async fn send_message(&self, system: &str, user_message: &str) -> Result<String, ClaudeError> {
        let request = MessageRequest {
            model: self.model.clone(),
            max_tokens: 4096,
            system: Some(system.to_string()),
            messages: vec![Message {
                role: "user".to_string(),
                content: user_message.to_string(),
            }],
        };

        let response = self.client
            .post(ANTHROPIC_API_URL)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| ClaudeError::RequestFailed(e.to_string()))?;

        let status = response.status();
        
        if status == 429 {
            return Err(ClaudeError::RateLimited);
        }

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ClaudeError::ApiError(format!("{}: {}", status, error_text)));
        }

        let response_body: MessageResponse = response
            .json()
            .await
            .map_err(|e| ClaudeError::ParseError(e.to_string()))?;

        // Extract text from content blocks
        let text = response_body
            .content
            .into_iter()
            .filter_map(|block| {
                if block.content_type == "text" {
                    block.text
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        Ok(text)
    }

    /// Send a message and stream the response
    pub async fn send_message_streaming(
        &self,
        system: &str,
        user_message: &str,
    ) -> Result<mpsc::Receiver<String>, ClaudeError> {
        let (tx, rx) = mpsc::channel(100);

        let request = MessageRequest {
            model: self.model.clone(),
            max_tokens: 4096,
            system: Some(system.to_string()),
            messages: vec![Message {
                role: "user".to_string(),
                content: user_message.to_string(),
            }],
        };

        let client = self.client.clone();
        let api_key = self.api_key.clone();

        tokio::spawn(async move {
            let response = match client
                .post(ANTHROPIC_API_URL)
                .header("x-api-key", &api_key)
                .header("anthropic-version", ANTHROPIC_VERSION)
                .header("content-type", "application/json")
                .json(&request)
                .send()
                .await
            {
                Ok(r) => r,
                Err(e) => {
                    let _ = tx.send(format!("Error: {}", e)).await;
                    return;
                }
            };

            if !response.status().is_success() {
                let error = response.text().await.unwrap_or_default();
                let _ = tx.send(format!("API Error: {}", error)).await;
                return;
            }

            // For non-streaming, just parse and send the full response
            match response.json::<MessageResponse>().await {
                Ok(body) => {
                    for block in body.content {
                        if block.content_type == "text" {
                            if let Some(text) = block.text {
                                let _ = tx.send(text).await;
                            }
                        }
                    }
                }
                Err(e) => {
                    let _ = tx.send(format!("Parse error: {}", e)).await;
                }
            }
        });

        Ok(rx)
    }

    /// Send a message expecting JSON output
    pub async fn send_json_message<T: for<'de> Deserialize<'de>>(
        &self,
        system: &str,
        user_message: &str,
    ) -> Result<T, ClaudeError> {
        let response = self.send_message(system, user_message).await?;
        
        // Try to extract JSON from the response
        let json_str = extract_json(&response)
            .ok_or_else(|| ClaudeError::ParseError("No JSON found in response".to_string()))?;

        serde_json::from_str(json_str)
            .map_err(|e| ClaudeError::ParseError(format!("JSON parse error: {}", e)))
    }

    /// Check if API key is configured
    pub fn is_configured() -> bool {
        Self::get_api_key().is_ok()
    }
}

/// Extract JSON from a response that may contain markdown code blocks
fn extract_json(text: &str) -> Option<&str> {
    // Try to find JSON in code block
    if let Some(start) = text.find("```json") {
        let json_start = start + 7;
        if let Some(end) = text[json_start..].find("```") {
            return Some(text[json_start..json_start + end].trim());
        }
    }

    // Try plain code block
    if let Some(start) = text.find("```") {
        let code_start = start + 3;
        // Skip language identifier if present
        let content_start = text[code_start..]
            .find('\n')
            .map(|i| code_start + i + 1)
            .unwrap_or(code_start);
        
        if let Some(end) = text[content_start..].find("```") {
            return Some(text[content_start..content_start + end].trim());
        }
    }

    // Try to find raw JSON object
    if let Some(start) = text.find('{') {
        // Find matching closing brace
        let mut depth = 0;
        let mut end = start;
        for (i, c) in text[start..].char_indices() {
            match c {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        end = start + i + 1;
                        break;
                    }
                }
                _ => {}
            }
        }
        if depth == 0 && end > start {
            return Some(&text[start..end]);
        }
    }

    None
}

/// API request structure
#[derive(Debug, Serialize)]
struct MessageRequest {
    model: String,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    messages: Vec<Message>,
}

#[derive(Debug, Serialize)]
struct Message {
    role: String,
    content: String,
}

/// API response structure
#[derive(Debug, Deserialize)]
struct MessageResponse {
    content: Vec<ContentBlock>,
    #[allow(dead_code)]
    model: String,
    #[allow(dead_code)]
    stop_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    content_type: String,
    text: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_json_code_block() {
        let text = r#"Here's the recommendation:
```json
{"setting": "value"}
```
Hope this helps!"#;
        
        assert_eq!(extract_json(text), Some(r#"{"setting": "value"}"#));
    }

    #[test]
    fn test_extract_json_raw() {
        let text = r#"The settings are: {"fps_limit": 60, "vsync": true}"#;
        assert_eq!(extract_json(text), Some(r#"{"fps_limit": 60, "vsync": true}"#));
    }
}
