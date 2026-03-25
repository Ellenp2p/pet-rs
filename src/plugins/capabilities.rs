//! Capability 模型
//!
//! 插件注册到特定的能力类型。

use crate::error::FrameworkError;
use std::collections::HashMap;

/// 能力类型
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Capability {
    /// 决策能力
    Decision,
    /// 记忆能力
    Memory,
    /// 工具调用能力
    Tool,
    /// 天气查询
    Weather,
    /// 日历操作
    Calendar,
    /// 文件读写
    FileIO,
    /// 网络搜索
    WebSearch,
    /// 代码执行
    CodeExecution,
    /// 发送消息
    SendMessage,
    /// 接收消息
    ReceiveMessage,
    /// 自定义能力
    Custom(String),
}

impl Capability {
    /// 获取能力名称
    pub fn name(&self) -> &str {
        match self {
            Capability::Decision => "decision",
            Capability::Memory => "memory",
            Capability::Tool => "tool",
            Capability::Weather => "weather",
            Capability::Calendar => "calendar",
            Capability::FileIO => "file_io",
            Capability::WebSearch => "web_search",
            Capability::CodeExecution => "code_execution",
            Capability::SendMessage => "send_message",
            Capability::ReceiveMessage => "receive_message",
            Capability::Custom(name) => name,
        }
    }

    /// 从名称解析能力
    pub fn from_name(name: &str) -> Option<Capability> {
        match name {
            "decision" => Some(Capability::Decision),
            "memory" => Some(Capability::Memory),
            "tool" => Some(Capability::Tool),
            "weather" => Some(Capability::Weather),
            "calendar" => Some(Capability::Calendar),
            "file_io" => Some(Capability::FileIO),
            "web_search" => Some(Capability::WebSearch),
            "code_execution" => Some(Capability::CodeExecution),
            "send_message" => Some(Capability::SendMessage),
            "receive_message" => Some(Capability::ReceiveMessage),
            _ => Some(Capability::Custom(name.to_string())),
        }
    }
}

/// 能力提供者
#[derive(Debug, Clone)]
pub struct CapabilityProvider {
    /// 插件 ID
    pub plugin_id: String,
    /// 是否启用
    pub enabled: bool,
    /// 优先级（数字越小优先级越高）
    pub priority: i32,
}

/// 能力注册表
pub struct CapabilityRegistry {
    /// 能力提供者映射
    providers: HashMap<Capability, Vec<CapabilityProvider>>,
}

impl CapabilityRegistry {
    /// 创建新的能力注册表
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    /// 注册能力
    pub fn register(
        &mut self,
        capability: Capability,
        plugin_id: String,
        priority: i32,
    ) -> Result<(), FrameworkError> {
        let provider = CapabilityProvider {
            plugin_id,
            enabled: true,
            priority,
        };

        // 克隆 capability 以便在 entry 之后还能使用
        let capability_clone = capability.clone();

        self.providers.entry(capability).or_default().push(provider);

        // 按优先级排序
        if let Some(providers) = self.providers.get_mut(&capability_clone) {
            providers.sort_by_key(|p| p.priority);
        }

        Ok(())
    }

    /// 注销能力
    pub fn unregister(
        &mut self,
        capability: &Capability,
        plugin_id: &str,
    ) -> Result<(), FrameworkError> {
        if let Some(providers) = self.providers.get_mut(capability) {
            providers.retain(|p| p.plugin_id != plugin_id);
        }
        Ok(())
    }

    /// 获取能力的提供者
    pub fn get_providers(&self, capability: &Capability) -> &[CapabilityProvider] {
        self.providers
            .get(capability)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// 获取启用的提供者
    pub fn get_enabled_providers(&self, capability: &Capability) -> Vec<&CapabilityProvider> {
        self.get_providers(capability)
            .iter()
            .filter(|p| p.enabled)
            .collect()
    }

    /// 获取最佳提供者（优先级最高且启用的）
    pub fn get_best_provider(&self, capability: &Capability) -> Option<&CapabilityProvider> {
        self.get_enabled_providers(capability).first().copied()
    }

    /// 启用提供者
    pub fn enable(
        &mut self,
        capability: &Capability,
        plugin_id: &str,
    ) -> Result<(), FrameworkError> {
        if let Some(providers) = self.providers.get_mut(capability) {
            for provider in providers.iter_mut() {
                if provider.plugin_id == plugin_id {
                    provider.enabled = true;
                    return Ok(());
                }
            }
        }
        Err(FrameworkError::Other(format!(
            "Provider '{}' not found for capability '{}'",
            plugin_id,
            capability.name()
        )))
    }

    /// 禁用提供者
    pub fn disable(
        &mut self,
        capability: &Capability,
        plugin_id: &str,
    ) -> Result<(), FrameworkError> {
        if let Some(providers) = self.providers.get_mut(capability) {
            for provider in providers.iter_mut() {
                if provider.plugin_id == plugin_id {
                    provider.enabled = false;
                    return Ok(());
                }
            }
        }
        Err(FrameworkError::Other(format!(
            "Provider '{}' not found for capability '{}'",
            plugin_id,
            capability.name()
        )))
    }

    /// 检查能力是否有提供者
    pub fn has_provider(&self, capability: &Capability) -> bool {
        self.providers
            .get(capability)
            .map(|v| !v.is_empty())
            .unwrap_or(false)
    }

    /// 获取所有已注册的能力
    pub fn registered_capabilities(&self) -> Vec<&Capability> {
        self.providers.keys().collect()
    }
}

impl Default for CapabilityRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_registry() {
        let mut registry = CapabilityRegistry::new();

        assert!(registry
            .register(Capability::Tool, "plugin-1".to_string(), 100)
            .is_ok());
        assert!(registry
            .register(Capability::Tool, "plugin-2".to_string(), 50)
            .is_ok());

        let providers = registry.get_enabled_providers(&Capability::Tool);
        assert_eq!(providers.len(), 2);
        assert_eq!(providers[0].plugin_id, "plugin-2"); // 优先级更高
    }

    #[test]
    fn test_capability_from_name() {
        assert_eq!(Capability::from_name("tool"), Some(Capability::Tool));
        assert_eq!(
            Capability::from_name("custom"),
            Some(Capability::Custom("custom".to_string()))
        );
    }
}
