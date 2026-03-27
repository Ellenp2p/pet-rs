//! 配置管理模块

use crate::ai::budget::BudgetConfig;
use crate::ai::provider::ProviderConfig;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 应用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub settings: Settings,
    pub ai: AIConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub memory_path: PathBuf,
    pub window_width: u16,
    pub window_height: u16,
    pub animation_speed: u64,
    pub dog_size: DogSize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIConfig {
    pub auto_switch: bool,
    pub switch_order: Vec<String>,
    pub providers: Vec<ProviderConfig>,
    pub budget: BudgetConfig,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum DogSize {
    Small,
    Medium,
    Large,
}

impl Default for Config {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_default();
        Self {
            settings: Settings {
                memory_path: home.join(".pet_agent").join("memory.json"),
                window_width: 80,
                window_height: 24,
                animation_speed: 200,
                dog_size: DogSize::Medium,
            },
            ai: AIConfig {
                auto_switch: true,
                switch_order: vec![
                    "openrouter".to_string(),
                    "openai".to_string(),
                    "ollama".to_string(),
                ],
                providers: Vec::new(),
                budget: BudgetConfig::default(),
            },
        }
    }
}

impl Config {
    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_default()
            .join("pet_agent")
            .join("config.toml")
    }

    pub fn load() -> anyhow::Result<Self> {
        let path = Self::config_path();
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let config: Self = toml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    pub fn needs_setup(&self) -> bool {
        self.ai.providers.is_empty()
            || self
                .ai
                .providers
                .iter()
                .all(|p: &ProviderConfig| p.api_key().is_empty() || !p.enabled)
    }

    pub fn memory_path(&self) -> PathBuf {
        self.settings.memory_path.clone()
    }

    pub fn usage_path(&self) -> PathBuf {
        dirs::home_dir()
            .unwrap_or_default()
            .join(".pet_agent")
            .join("usage.json")
    }
}
