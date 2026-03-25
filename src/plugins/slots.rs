//! Slot 系统
//!
//! Slot 是独占的，只有一个插件可以接管某个功能。

use crate::error::FrameworkError;
use std::collections::HashMap;

/// Slot 类型
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Slot {
    /// 决策引擎（独占）
    DecisionEngine,
    /// 记忆提供者（独占）
    MemoryProvider,
    /// LLM 提供者（可配置）
    LLMProvider,
    /// 输出格式化（独占）
    OutputFormatter,
    /// 自定义 Slot
    Custom(String),
}

impl Slot {
    /// 获取 Slot 名称
    pub fn name(&self) -> &str {
        match self {
            Slot::DecisionEngine => "decision_engine",
            Slot::MemoryProvider => "memory_provider",
            Slot::LLMProvider => "llm_provider",
            Slot::OutputFormatter => "output_formatter",
            Slot::Custom(name) => name,
        }
    }

    /// 从名称解析 Slot
    pub fn from_name(name: &str) -> Option<Slot> {
        match name {
            "decision_engine" => Some(Slot::DecisionEngine),
            "memory_provider" => Some(Slot::MemoryProvider),
            "llm_provider" => Some(Slot::LLMProvider),
            "output_formatter" => Some(Slot::OutputFormatter),
            _ => Some(Slot::Custom(name.to_string())),
        }
    }

    /// 是否是独占的
    pub fn is_exclusive(&self) -> bool {
        match self {
            Slot::DecisionEngine => true,
            Slot::MemoryProvider => true,
            Slot::LLMProvider => false,
            Slot::OutputFormatter => true,
            Slot::Custom(_) => true,
        }
    }
}

/// Slot 注册信息
#[derive(Debug, Clone)]
pub struct SlotRegistration {
    /// Slot 类型
    pub slot: Slot,
    /// 插件 ID
    pub plugin_id: String,
    /// 是否启用
    pub enabled: bool,
}

/// Slot 管理器
pub struct SlotManager {
    /// Slot 注册信息
    registrations: HashMap<Slot, SlotRegistration>,
}

impl SlotManager {
    /// 创建新的 Slot 管理器
    pub fn new() -> Self {
        Self {
            registrations: HashMap::new(),
        }
    }

    /// 注册 Slot
    pub fn register(&mut self, slot: Slot, plugin_id: String) -> Result<(), FrameworkError> {
        // 检查是否已注册
        if let Some(existing) = self.registrations.get(&slot) {
            if slot.is_exclusive() {
                return Err(FrameworkError::Other(format!(
                    "Slot '{}' is already registered by plugin '{}'",
                    slot.name(),
                    existing.plugin_id
                )));
            }
        }

        self.registrations.insert(
            slot.clone(),
            SlotRegistration {
                slot,
                plugin_id,
                enabled: true,
            },
        );

        Ok(())
    }

    /// 注销 Slot
    pub fn unregister(&mut self, slot: &Slot, plugin_id: &str) -> Result<(), FrameworkError> {
        if let Some(existing) = self.registrations.get(slot) {
            if existing.plugin_id != plugin_id {
                return Err(FrameworkError::Other(format!(
                    "Slot '{}' is registered by plugin '{}', not '{}'",
                    slot.name(),
                    existing.plugin_id,
                    plugin_id
                )));
            }
        }

        self.registrations.remove(slot);
        Ok(())
    }

    /// 获取 Slot 的插件 ID
    pub fn get(&self, slot: &Slot) -> Option<&str> {
        self.registrations.get(slot).map(|r| r.plugin_id.as_str())
    }

    /// 检查 Slot 是否已注册
    pub fn is_registered(&self, slot: &Slot) -> bool {
        self.registrations.contains_key(slot)
    }

    /// 启用 Slot
    pub fn enable(&mut self, slot: &Slot) -> Result<(), FrameworkError> {
        if let Some(registration) = self.registrations.get_mut(slot) {
            registration.enabled = true;
            Ok(())
        } else {
            Err(FrameworkError::Other(format!(
                "Slot '{}' is not registered",
                slot.name()
            )))
        }
    }

    /// 禁用 Slot
    pub fn disable(&mut self, slot: &Slot) -> Result<(), FrameworkError> {
        if let Some(registration) = self.registrations.get_mut(slot) {
            registration.enabled = false;
            Ok(())
        } else {
            Err(FrameworkError::Other(format!(
                "Slot '{}' is not registered",
                slot.name()
            )))
        }
    }

    /// 获取所有已注册的 Slot
    pub fn registered_slots(&self) -> Vec<&Slot> {
        self.registrations.keys().collect()
    }

    /// 获取所有已启用的 Slot
    pub fn enabled_slots(&self) -> Vec<&Slot> {
        self.registrations
            .iter()
            .filter(|(_, r)| r.enabled)
            .map(|(slot, _)| slot)
            .collect()
    }
}

impl Default for SlotManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slot_manager() {
        let mut manager = SlotManager::new();

        assert!(manager
            .register(Slot::DecisionEngine, "plugin-1".to_string())
            .is_ok());
        assert_eq!(manager.get(&Slot::DecisionEngine), Some("plugin-1"));

        // 独占 Slot 不能重复注册
        assert!(manager
            .register(Slot::DecisionEngine, "plugin-2".to_string())
            .is_err());
    }

    #[test]
    fn test_slot_from_name() {
        assert_eq!(
            Slot::from_name("decision_engine"),
            Some(Slot::DecisionEngine)
        );
        assert_eq!(
            Slot::from_name("custom_slot"),
            Some(Slot::Custom("custom_slot".to_string()))
        );
    }
}
