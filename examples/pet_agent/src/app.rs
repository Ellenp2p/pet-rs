//! 应用状态管理模块

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use agent_pet_rs::prelude::AIProviderManager;

use crate::config::AppConfig;
use crate::pet::Pet;

const MAX_HISTORY: usize = 50;

/// 对话历史条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub role: String,
    pub content: String,
    pub timestamp: u64,
}

/// 对话历史
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationHistory {
    pub entries: Vec<HistoryEntry>,
}

impl Default for ConversationHistory {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
}

impl ConversationHistory {
    pub fn push(&mut self, role: &str, content: &str) {
        self.entries.push(HistoryEntry {
            role: role.to_string(),
            content: content.to_string(),
            timestamp: current_timestamp(),
        });

        // 保持最多 MAX_HISTORY 条
        if self.entries.len() > MAX_HISTORY {
            self.entries.remove(0);
        }
    }

    pub fn to_chat_messages(&self) -> Vec<agent_pet_rs::prelude::ChatMessage> {
        self.entries
            .iter()
            .map(|e| agent_pet_rs::prelude::ChatMessage {
                role: e.role.clone(),
                content: e.content.clone(),
            })
            .collect()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

/// 应用状态
pub struct AppState {
    pub pet: Pet,
    pub history: ConversationHistory,
    pub ai: AIProviderManager,
    pub config: AppConfig,
    pub running: bool,
    pub input: String,
    pub messages: Vec<String>, // 显示的消息列表
}

impl AppState {
    /// 加载或创建默认状态
    pub fn load_or_default(
        ai: AIProviderManager,
        config: AppConfig,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let pet = Self::load_pet(&config.pet.name).unwrap_or_else(|_| Pet::new(&config.pet.name));
        let history = Self::load_history().unwrap_or_default();

        let messages: Vec<String> = history
            .entries
            .iter()
            .map(|e| {
                if e.role == "user" {
                    format!("> {}", e.content)
                } else {
                    format!("< {}", e.content)
                }
            })
            .collect();

        Ok(Self {
            pet,
            history,
            ai,
            config,
            running: true,
            input: String::new(),
            messages,
        })
    }

    /// 保存状态
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        Self::save_pet(&self.pet)?;
        Self::save_history(&self.history)?;
        Ok(())
    }

    fn data_dir() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".pet_agent")
    }

    fn load_pet(name: &str) -> Result<Pet, Box<dyn std::error::Error>> {
        let path = Self::data_dir().join("pet_state.json");
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let mut pet: Pet = serde_json::from_str(&content)?;
            pet.update(); // 加载后立即更新状态
            Ok(pet)
        } else {
            Ok(Pet::new(name))
        }
    }

    fn save_pet(pet: &Pet) -> Result<(), Box<dyn std::error::Error>> {
        let dir = Self::data_dir();
        std::fs::create_dir_all(&dir)?;
        let path = dir.join("pet_state.json");
        let content = serde_json::to_string_pretty(pet)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    fn load_history() -> Result<ConversationHistory, Box<dyn std::error::Error>> {
        let path = Self::data_dir().join("history.json");
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let history: ConversationHistory = serde_json::from_str(&content)?;
            Ok(history)
        } else {
            Ok(ConversationHistory::default())
        }
    }

    fn save_history(history: &ConversationHistory) -> Result<(), Box<dyn std::error::Error>> {
        let dir = Self::data_dir();
        std::fs::create_dir_all(&dir)?;
        let path = dir.join("history.json");
        let content = serde_json::to_string_pretty(history)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// 添加用户消息
    pub fn add_user_message(&mut self, content: &str) {
        self.history.push("user", content);
        self.messages.push(format!("> {}", content));
    }

    /// 添加助手消息
    pub fn add_assistant_message(&mut self, content: &str) {
        self.history.push("assistant", content);
        self.messages.push(format!("< {}", content));
    }

    /// 添加系统消息
    pub fn add_system_message(&mut self, content: &str) {
        self.messages.push(format!("  [{}]", content));
    }
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
