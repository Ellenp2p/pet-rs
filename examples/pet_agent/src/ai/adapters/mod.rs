//! AI 提供商适配器

use crate::ai::provider::{ChatMessage, ChatResponse, ProviderConfig, TokenUsage};
use crate::ai::error::AIError;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    max_tokens: u32,
    temperature: f32,
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

pub async fn chat(messages: Vec<ChatMessage>, config: &ProviderConfig) -> Result<ChatResponse, AIError> {
    let client = reqwest::Client::new();
    let api_base = config.api_base();
    let request = ChatRequest {
        model: config.model.clone(),
        messages,
        max_tokens: config.max_tokens.unwrap_or(1000),
        temperature: config.temperature.unwrap_or(0.7),
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

    let inner: ChatResponseInner = response.json().await
        .map_err(|e| AIError::ParseError(e.to_string()))?;

    let choice = inner.choices.into_iter().next()
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
