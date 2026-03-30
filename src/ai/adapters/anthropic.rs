//! Anthropic (Claude) 适配器

use crate::ai::error::AIError;
use crate::ai::provider::{
    AIProvider, ChatMessage, ChatResponse, ProviderConfig, ProviderType, TokenUsage,
};
use serde::{Deserialize, Serialize};

/// Anthropic 提供商
pub struct AnthropicProvider;

impl Default for AnthropicProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl AnthropicProvider {
    pub fn new() -> Self {
        Self
    }
}

/// Anthropic 消息格式（不包含 system）
#[derive(Serialize, Deserialize, Clone)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct MessagesRequest {
    model: String,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

#[derive(Deserialize)]
struct MessagesResponse {
    content: Vec<ContentBlock>,
    usage: UsageInner,
    stop_reason: Option<String>,
}

#[derive(Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    block_type: String,
    text: Option<String>,
}

#[derive(Deserialize)]
struct UsageInner {
    input_tokens: u32,
    output_tokens: u32,
}

#[derive(Deserialize)]
struct StreamEvent {
    #[serde(rename = "type")]
    event_type: String,
    #[serde(default)]
    delta: Option<DeltaBlock>,
    #[serde(default)]
    #[allow(dead_code)]
    usage: Option<UsageInner>,
    #[serde(default)]
    stop_reason: Option<String>,
}

#[derive(Deserialize)]
struct DeltaBlock {
    #[serde(rename = "type")]
    delta_type: String,
    text: Option<String>,
}

impl AIProvider for AnthropicProvider {
    fn name(&self) -> &str {
        "anthropic"
    }

    fn provider_type(&self) -> ProviderType {
        ProviderType::Anthropic
    }

    fn supported_models(&self) -> Vec<String> {
        vec![
            "claude-3-opus-20240229".to_string(),
            "claude-3-sonnet-20240229".to_string(),
            "claude-3-haiku-20240307".to_string(),
        ]
    }

    fn chat(
        &self,
        messages: Vec<ChatMessage>,
        config: &ProviderConfig,
    ) -> Result<ChatResponse, AIError> {
        let rt =
            tokio::runtime::Runtime::new().map_err(|e| AIError::NetworkError(e.to_string()))?;

        rt.block_on(self.chat_async(messages, config))
    }

    fn chat_stream(
        &self,
        messages: Vec<ChatMessage>,
        config: &ProviderConfig,
        on_chunk: Box<dyn Fn(String) + Send>,
    ) -> Result<ChatResponse, AIError> {
        let rt =
            tokio::runtime::Runtime::new().map_err(|e| AIError::NetworkError(e.to_string()))?;

        rt.block_on(self.chat_stream_async(messages, config, on_chunk))
    }
}

impl AnthropicProvider {
    /// 将通用 ChatMessage 转换为 Anthropic 格式（分离 system 消息）
    fn convert_messages(messages: Vec<ChatMessage>) -> (Option<String>, Vec<AnthropicMessage>) {
        let mut system = None;
        let mut anthropic_messages = Vec::new();

        for msg in messages {
            if msg.role == "system" {
                system = Some(msg.content);
            } else {
                anthropic_messages.push(AnthropicMessage {
                    role: msg.role,
                    content: msg.content,
                });
            }
        }

        (system, anthropic_messages)
    }

    async fn chat_async(
        &self,
        messages: Vec<ChatMessage>,
        config: &ProviderConfig,
    ) -> Result<ChatResponse, AIError> {
        let client = reqwest::Client::new();
        let api_base = config.api_base();
        let (system, anthropic_messages) = Self::convert_messages(messages);

        let request = MessagesRequest {
            model: config.model.clone(),
            max_tokens: config.max_tokens.unwrap_or(1000),
            system,
            messages: anthropic_messages,
            stream: None,
        };

        let response = client
            .post(format!("{}/messages", api_base))
            .header("x-api-key", config.api_key())
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| AIError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(AIError::ApiError(format!("{}: {}", status, text)));
        }

        let inner: MessagesResponse = response
            .json()
            .await
            .map_err(|e| AIError::ParseError(e.to_string()))?;

        // 提取文本内容
        let content = inner
            .content
            .into_iter()
            .filter(|b| b.block_type == "text")
            .filter_map(|b| b.text)
            .collect::<Vec<_>>()
            .join("");

        Ok(ChatResponse {
            content,
            usage: TokenUsage {
                input_tokens: inner.usage.input_tokens,
                output_tokens: inner.usage.output_tokens,
                total_tokens: inner.usage.input_tokens + inner.usage.output_tokens,
            },
            model: config.model.clone(),
            finish_reason: inner.stop_reason.unwrap_or_default(),
        })
    }

    async fn chat_stream_async(
        &self,
        messages: Vec<ChatMessage>,
        config: &ProviderConfig,
        on_chunk: Box<dyn Fn(String) + Send>,
    ) -> Result<ChatResponse, AIError> {
        let client = reqwest::Client::new();
        let api_base = config.api_base();
        let (system, anthropic_messages) = Self::convert_messages(messages);

        let request = MessagesRequest {
            model: config.model.clone(),
            max_tokens: config.max_tokens.unwrap_or(1000),
            system,
            messages: anthropic_messages,
            stream: Some(true),
        };

        let response = client
            .post(format!("{}/messages", api_base))
            .header("x-api-key", config.api_key())
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| AIError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(AIError::ApiError(format!("{}: {}", status, text)));
        }

        let body = response
            .text()
            .await
            .map_err(|e| AIError::NetworkError(e.to_string()))?;

        let mut full_content = String::new();
        let total_input_tokens = 0u32;
        let total_output_tokens = 0u32;
        let mut finish_reason = String::new();

        for line in body.lines() {
            let line = line.trim();
            if line.starts_with("data: ") {
                let data = line.strip_prefix("data: ").unwrap_or("");
                if let Ok(event) = serde_json::from_str::<StreamEvent>(data) {
                    match event.event_type.as_str() {
                        "content_block_delta" => {
                            if let Some(delta) = event.delta {
                                if delta.delta_type == "text_delta" {
                                    if let Some(text) = delta.text {
                                        full_content.push_str(&text);
                                        on_chunk(text);
                                    }
                                }
                            }
                        }
                        "message_delta" => {
                            if let Some(reason) = event.stop_reason {
                                finish_reason = reason;
                            }
                        }
                        "message_start" | "message_stop" => {
                            // 可以在这里处理 usage
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(ChatResponse {
            content: full_content,
            usage: TokenUsage {
                input_tokens: total_input_tokens,
                output_tokens: total_output_tokens,
                total_tokens: total_input_tokens + total_output_tokens,
            },
            model: config.model.clone(),
            finish_reason,
        })
    }
}
