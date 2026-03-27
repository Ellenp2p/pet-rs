//! AI 提供商接口

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone)]
pub struct ChatResponse {
    pub content: String,
    pub usage: TokenUsage,
    pub model: String,
    pub finish_reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProviderType {
    OpenRouter,
    OpenAI,
    Ollama,
    Custom,
}

impl ProviderType {
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "openrouter" => Some(Self::OpenRouter),
            "openai" => Some(Self::OpenAI),
            "ollama" => Some(Self::Ollama),
            "custom" => Some(Self::Custom),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::OpenRouter => "openrouter",
            Self::OpenAI => "openai",
            Self::Ollama => "ollama",
            Self::Custom => "custom",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::OpenRouter => "OpenRouter",
            Self::OpenAI => "OpenAI",
            Self::Ollama => "Ollama",
            Self::Custom => "Custom",
        }
    }

    pub fn default_model(&self) -> &'static str {
        match self {
            Self::OpenRouter => "google/gemma-3-27b-it:free",
            Self::OpenAI => "gpt-4o-mini",
            Self::Ollama => "llama3",
            Self::Custom => "",
        }
    }

    pub fn default_api_base(&self) -> &'static str {
        match self {
            Self::OpenRouter => "https://openrouter.ai/api/v1",
            Self::OpenAI => "https://api.openai.com/v1",
            Self::Ollama => "http://localhost:11434",
            Self::Custom => "",
        }
    }

    pub fn all() -> Vec<ProviderType> {
        vec![Self::OpenRouter, Self::OpenAI, Self::Ollama, Self::Custom]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub enabled: bool,
    pub requests_per_minute: Option<u32>,
    pub requests_per_hour: Option<u32>,
    pub tokens_per_minute: Option<u32>,
    pub tokens_per_hour: Option<u32>,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            requests_per_minute: Some(20),
            requests_per_hour: Some(200),
            tokens_per_minute: Some(40000),
            tokens_per_hour: Some(400000),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub name: String,
    pub enabled: bool,
    pub api_key_encrypted: String,
    pub model: String,
    pub api_base: Option<String>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub priority: u32,
    #[serde(default)]
    pub rate_limit: RateLimitConfig,
}

impl ProviderConfig {
    pub fn new(provider_type: ProviderType, api_key: &str) -> Self {
        Self {
            name: provider_type.name().to_string(),
            enabled: true,
            api_key_encrypted: super::crypto::encrypt_api_key(api_key),
            model: provider_type.default_model().to_string(),
            api_base: Some(provider_type.default_api_base().to_string()),
            max_tokens: Some(1000),
            temperature: Some(0.7),
            priority: 1,
            rate_limit: RateLimitConfig::default(),
        }
    }

    pub fn api_key(&self) -> String {
        super::crypto::decrypt_api_key(&self.api_key_encrypted)
    }

    pub fn api_base(&self) -> &str {
        self.api_base.as_deref().unwrap_or("")
    }
}
