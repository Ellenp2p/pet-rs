//! AI 提供商接口定义

use super::AIError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 聊天消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// Token 使用量
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub total_tokens: u32,
}

/// 聊天响应
#[derive(Debug, Clone)]
pub struct ChatResponse {
    pub content: String,
    pub usage: TokenUsage,
    pub model: String,
    pub finish_reason: String,
}

/// 提供商类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProviderType {
    OpenAI,
    Anthropic,
    OpenRouter,
    Google,
    Mistral,
    Cohere,
    Groq,
    Together,
    Ollama,
    LMStudio,
    Custom,
}

impl ProviderType {
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "openai" => Some(Self::OpenAI),
            "anthropic" | "claude" => Some(Self::Anthropic),
            "openrouter" => Some(Self::OpenRouter),
            "google" | "gemini" => Some(Self::Google),
            "mistral" => Some(Self::Mistral),
            "cohere" => Some(Self::Cohere),
            "groq" => Some(Self::Groq),
            "together" => Some(Self::Together),
            "ollama" => Some(Self::Ollama),
            "lmstudio" => Some(Self::LMStudio),
            "custom" => Some(Self::Custom),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::OpenAI => "openai",
            Self::Anthropic => "anthropic",
            Self::OpenRouter => "openrouter",
            Self::Google => "google",
            Self::Mistral => "mistral",
            Self::Cohere => "cohere",
            Self::Groq => "groq",
            Self::Together => "together",
            Self::Ollama => "ollama",
            Self::LMStudio => "lmstudio",
            Self::Custom => "custom",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::OpenAI => "OpenAI",
            Self::Anthropic => "Anthropic (Claude)",
            Self::OpenRouter => "OpenRouter",
            Self::Google => "Google (Gemini)",
            Self::Mistral => "Mistral",
            Self::Cohere => "Cohere",
            Self::Groq => "Groq",
            Self::Together => "Together AI",
            Self::Ollama => "Ollama",
            Self::LMStudio => "LM Studio",
            Self::Custom => "Custom",
        }
    }

    pub fn default_model(&self) -> &'static str {
        match self {
            Self::OpenAI => "gpt-4o-mini",
            Self::Anthropic => "claude-3-haiku-20240307",
            Self::OpenRouter => "stepfun/step-3.5-flash:free",
            Self::Google => "gemini-1.5-flash",
            Self::Mistral => "mistral-small-latest",
            Self::Cohere => "command-r",
            Self::Groq => "llama3-8b-8192",
            Self::Together => "meta-llama/Llama-3-8b-chat-hf",
            Self::Ollama => "llama3",
            Self::LMStudio => "",
            Self::Custom => "",
        }
    }

    pub fn default_api_base(&self) -> &'static str {
        match self {
            Self::OpenAI => "https://api.openai.com/v1",
            Self::Anthropic => "https://api.anthropic.com/v1",
            Self::OpenRouter => "https://openrouter.ai/api/v1",
            Self::Google => "https://generativelanguage.googleapis.com/v1",
            Self::Mistral => "https://api.mistral.ai/v1",
            Self::Cohere => "https://api.cohere.ai/v1",
            Self::Groq => "https://api.groq.com/openai/v1",
            Self::Together => "https://api.together.xyz/v1",
            Self::Ollama => "http://localhost:11434",
            Self::LMStudio => "http://localhost:1234/v1",
            Self::Custom => "",
        }
    }

    pub fn requires_api_key(&self) -> bool {
        !matches!(self, Self::Ollama | Self::LMStudio)
    }

    pub fn all() -> Vec<ProviderType> {
        vec![
            Self::OpenAI,
            Self::Anthropic,
            Self::OpenRouter,
            Self::Google,
            Self::Mistral,
            Self::Cohere,
            Self::Groq,
            Self::Together,
            Self::Ollama,
            Self::LMStudio,
            Self::Custom,
        ]
    }
}

/// 速率限制配置
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

/// 价格配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingConfig {
    pub input_per_million: f64,
    pub output_per_million: f64,
}

impl Default for PricingConfig {
    fn default() -> Self {
        Self {
            input_per_million: 0.0,
            output_per_million: 0.0,
        }
    }
}

/// 提供商配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub name: String,
    pub provider_type: ProviderType,
    pub enabled: bool,
    pub api_key: Option<String>,
    pub model: String,
    pub api_base: Option<String>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub priority: u32,
    #[serde(default)]
    pub rate_limit: RateLimitConfig,
    #[serde(default)]
    pub pricing: PricingConfig,
    #[serde(default)]
    pub extra_headers: HashMap<String, String>,
}

impl ProviderConfig {
    pub fn new(provider_type: ProviderType, api_key: &str) -> Self {
        Self {
            name: provider_type.name().to_string(),
            provider_type,
            enabled: true,
            api_key: if api_key.is_empty() {
                None
            } else {
                Some(api_key.to_string())
            },
            model: provider_type.default_model().to_string(),
            api_base: Some(provider_type.default_api_base().to_string()),
            max_tokens: Some(1000),
            temperature: Some(0.7),
            priority: 1,
            rate_limit: RateLimitConfig::default(),
            pricing: PricingConfig::default(),
            extra_headers: HashMap::new(),
        }
    }

    pub fn api_base(&self) -> &str {
        self.api_base
            .as_deref()
            .unwrap_or(self.provider_type.default_api_base())
    }

    pub fn api_key(&self) -> &str {
        self.api_key.as_deref().unwrap_or("")
    }
}

/// AI 提供商 trait
pub trait AIProvider: Send + Sync {
    /// 获取提供商名称
    fn name(&self) -> &str;

    /// 获取提供商类型
    fn provider_type(&self) -> ProviderType;

    /// 获取支持的模型列表
    fn supported_models(&self) -> Vec<String>;

    /// 同步聊天
    fn chat(
        &self,
        messages: Vec<ChatMessage>,
        config: &ProviderConfig,
    ) -> Result<ChatResponse, AIError>;

    /// 流式聊天（通过回调返回片段）
    fn chat_stream(
        &self,
        messages: Vec<ChatMessage>,
        config: &ProviderConfig,
        on_chunk: Box<dyn Fn(String) + Send>,
    ) -> Result<ChatResponse, AIError>;

    /// 计算费用
    fn calculate_cost(&self, usage: &TokenUsage, config: &ProviderConfig) -> f64 {
        let input_cost =
            (usage.input_tokens as f64 / 1_000_000.0) * config.pricing.input_per_million;
        let output_cost =
            (usage.output_tokens as f64 / 1_000_000.0) * config.pricing.output_per_million;
        input_cost + output_cost
    }
}
