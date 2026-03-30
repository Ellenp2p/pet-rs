//! Token 计费系统

use super::provider::ProviderType;

/// 内置价格表
pub struct PricingTable;

impl PricingTable {
    /// 获取指定提供商和模型的价格 (input_per_million, output_per_million)
    pub fn get_prices(provider_type: ProviderType, model: &str) -> (f64, f64) {
        match provider_type {
            ProviderType::OpenAI => Self::openai_prices(model),
            ProviderType::Anthropic => Self::anthropic_prices(model),
            ProviderType::OpenRouter => Self::openrouter_prices(model),
            ProviderType::Google => Self::google_prices(model),
            ProviderType::Ollama | ProviderType::LMStudio => (0.0, 0.0),
            _ => (0.0, 0.0),
        }
    }

    fn openai_prices(model: &str) -> (f64, f64) {
        match model {
            "gpt-4o" => (2.50, 10.00),
            "gpt-4o-mini" => (0.15, 0.60),
            "gpt-4-turbo" => (10.00, 30.00),
            "gpt-3.5-turbo" => (0.50, 1.50),
            _ => (0.0, 0.0),
        }
    }

    fn anthropic_prices(model: &str) -> (f64, f64) {
        if model.contains("opus") {
            (15.00, 75.00)
        } else if model.contains("sonnet") {
            (3.00, 15.00)
        } else if model.contains("haiku") {
            (0.25, 1.25)
        } else {
            (0.0, 0.0)
        }
    }

    fn openrouter_prices(model: &str) -> (f64, f64) {
        if model.contains("free") {
            (0.0, 0.0)
        } else {
            (0.50, 0.50)
        }
    }

    fn google_prices(model: &str) -> (f64, f64) {
        if model.contains("pro") {
            (3.50, 10.50)
        } else if model.contains("flash") {
            (0.075, 0.30)
        } else {
            (0.0, 0.0)
        }
    }
}
