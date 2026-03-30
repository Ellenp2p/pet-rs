//! OpenAI 适配器

use crate::ai::error::AIError;
use crate::ai::provider::{
    AIProvider, ChatMessage, ChatResponse, ProviderConfig, ProviderType, TokenUsage,
};
use serde::{Deserialize, Serialize};

/// OpenAI 提供商
pub struct OpenAIProvider;

impl Default for OpenAIProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl OpenAIProvider {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    max_tokens: u32,
    temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

#[derive(Deserialize)]
struct ChatResponseInner {
    choices: Vec<Choice>,
    usage: Option<UsageInner>,
}

#[derive(Deserialize)]
struct Choice {
    message: ChatMessage,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct UsageInner {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Deserialize)]
struct SSEChunk {
    choices: Vec<SSEChoice>,
}

#[derive(Deserialize)]
struct SSEChoice {
    delta: SSEDelta,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct SSEDelta {
    content: Option<String>,
}

impl AIProvider for OpenAIProvider {
    fn name(&self) -> &str {
        "openai"
    }

    fn provider_type(&self) -> ProviderType {
        ProviderType::OpenAI
    }

    fn supported_models(&self) -> Vec<String> {
        vec![
            "gpt-4o".to_string(),
            "gpt-4o-mini".to_string(),
            "gpt-4-turbo".to_string(),
            "gpt-3.5-turbo".to_string(),
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

impl OpenAIProvider {
    async fn chat_async(
        &self,
        messages: Vec<ChatMessage>,
        config: &ProviderConfig,
    ) -> Result<ChatResponse, AIError> {
        let client = reqwest::Client::new();
        let api_base = config.api_base();
        let request = ChatRequest {
            model: config.model.clone(),
            messages,
            max_tokens: config.max_tokens.unwrap_or(1000),
            temperature: config.temperature.unwrap_or(0.7),
            stream: None,
        };

        let response = client
            .post(format!("{}/chat/completions", api_base))
            .header("Authorization", format!("Bearer {}", config.api_key()))
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

        let inner: ChatResponseInner = response
            .json()
            .await
            .map_err(|e| AIError::ParseError(e.to_string()))?;

        let choice = inner
            .choices
            .into_iter()
            .next()
            .ok_or(AIError::EmptyResponse)?;

        let usage = inner.usage.unwrap_or(UsageInner {
            prompt_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0,
        });

        Ok(ChatResponse {
            content: choice.message.content,
            usage: TokenUsage {
                input_tokens: usage.prompt_tokens,
                output_tokens: usage.completion_tokens,
                total_tokens: usage.total_tokens,
            },
            model: config.model.clone(),
            finish_reason: choice.finish_reason.unwrap_or_default(),
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

        let request = ChatRequest {
            model: config.model.clone(),
            messages,
            max_tokens: config.max_tokens.unwrap_or(1000),
            temperature: config.temperature.unwrap_or(0.7),
            stream: Some(true),
        };

        let response = client
            .post(format!("{}/chat/completions", api_base))
            .header("Authorization", format!("Bearer {}", config.api_key()))
            .header("Content-Type", "application/json")
            .header("Accept", "text/event-stream")
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
        let mut finish_reason: Option<String> = None;

        for line in body.lines() {
            let line = line.trim();
            if line.starts_with("data: ") {
                let data = line.strip_prefix("data: ").unwrap_or("");
                if data == "[DONE]" {
                    break;
                }
                if let Ok(chunk) = serde_json::from_str::<SSEChunk>(data) {
                    if let Some(choice) = chunk.choices.first() {
                        if let Some(content) = &choice.delta.content {
                            if !content.is_empty() {
                                full_content.push_str(content);
                                on_chunk(content.clone());
                            }
                        }
                        if choice.finish_reason.is_some() {
                            finish_reason = choice.finish_reason.clone();
                        }
                    }
                }
            }
        }

        Ok(ChatResponse {
            content: full_content,
            usage: TokenUsage {
                input_tokens: 0,
                output_tokens: 0,
                total_tokens: 0,
            },
            model: config.model.clone(),
            finish_reason: finish_reason.unwrap_or_default(),
        })
    }
}
