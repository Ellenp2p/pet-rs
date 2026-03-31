//! 简化配置模块

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use agent_pet_rs::prelude::{AIConfig, BudgetConfig, ProviderConfig, ProviderType};

/// 应用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub ai: AiConfig,
    pub pet: PetConfig,
}

/// AI 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    pub default_provider: String,
    pub providers: HashMap<String, AiProviderConfig>,
}

/// AI 提供商配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiProviderConfig {
    pub enabled: bool,
    pub api_key: String,
    pub model: String,
    #[serde(default)]
    pub api_base: Option<String>,
}

/// 宠物配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PetConfig {
    #[serde(default = "default_pet_name")]
    pub name: String,
}

fn default_pet_name() -> String {
    "Pet".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            ai: AiConfig {
                default_provider: "openai".to_string(),
                providers: HashMap::from([(
                    "openai".to_string(),
                    AiProviderConfig {
                        enabled: true,
                        api_key: String::new(),
                        model: "gpt-4o-mini".to_string(),
                        api_base: None,
                    },
                )]),
            },
            pet: PetConfig {
                name: "Pet".to_string(),
            },
        }
    }
}

impl AppConfig {
    /// 获取配置目录路径
    pub fn config_dir() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".pet_agent")
    }

    /// 获取配置文件路径
    pub fn config_path() -> PathBuf {
        Self::config_dir().join("config.toml")
    }

    /// 加载配置
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let path = Self::config_path();
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let config: AppConfig = toml::from_str(&content)?;
            Ok(config)
        } else {
            // 创建默认配置
            let config = Self::default();
            config.save()?;
            Ok(config)
        }
    }

    /// 保存配置
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let dir = Self::config_dir();
        std::fs::create_dir_all(&dir)?;

        let path = Self::config_path();
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&path, content)?;

        Ok(())
    }

    /// 转换为框架的 AIConfig
    pub fn to_ai_config(&self) -> AIConfig {
        let providers: HashMap<String, ProviderConfig> = self
            .ai
            .providers
            .iter()
            .filter(|(_, p)| p.enabled)
            .map(|(name, p)| {
                // OpenAI 兼容的提供商使用 OpenAI 适配器
                let provider_type = match name.to_lowercase().as_str() {
                    "openrouter" | "groq" | "together" => ProviderType::OpenAI,
                    other => ProviderType::from_name(other).unwrap_or(ProviderType::Custom),
                };
                let mut config = ProviderConfig::new(provider_type, &p.api_key);
                config.model = p.model.clone();

                // 根据提供商名称自动设置 api_base (如果配置中没有)
                if let Some(base) = &p.api_base {
                    config.api_base = Some(base.clone());
                } else {
                    config.api_base = Some(match name.to_lowercase().as_str() {
                        "openrouter" => "https://openrouter.ai/api/v1".to_string(),
                        "groq" => "https://api.groq.com/openai/v1".to_string(),
                        "together" => "https://api.together.xyz/v1".to_string(),
                        _ => provider_type.default_api_base().to_string(),
                    });
                }

                (name.clone(), config)
            })
            .collect();

        AIConfig {
            default_provider: self.ai.default_provider.clone(),
            auto_switch: true,
            switch_order: providers.keys().cloned().collect(),
            providers,
            budget: BudgetConfig::default(),
        }
    }
}
