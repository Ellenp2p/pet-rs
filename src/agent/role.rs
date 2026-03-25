//! Agent 角色系统
//!
//! 定义 Agent 的角色和能力。

use crate::agent::core::RoleConfig;
use crate::error::FrameworkError;
use serde::{Deserialize, Serialize};

/// 角色 trait
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleTrait {
    /// 特征名称
    pub name: String,
    /// 特征描述
    pub description: String,
}

/// 角色
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    /// 角色名称
    name: String,
    /// 角色描述
    description: String,
    /// 角色能力
    capabilities: Vec<String>,
    /// 角色特征
    traits: Vec<RoleTrait>,
}

impl Role {
    /// 从配置创建角色
    pub fn from_config(config: &RoleConfig) -> Result<Self, FrameworkError> {
        Ok(Self {
            name: config.name.clone(),
            description: config.description.clone(),
            capabilities: config.capabilities.clone(),
            traits: Vec::new(),
        })
    }

    /// 获取角色名称
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 获取角色描述
    pub fn description(&self) -> &str {
        &self.description
    }

    /// 获取角色能力
    pub fn capabilities(&self) -> &[String] {
        &self.capabilities
    }

    /// 获取角色特征
    pub fn traits(&self) -> &[RoleTrait] {
        &self.traits
    }

    /// 是否具有某能力
    pub fn has_capability(&self, capability: &str) -> bool {
        self.capabilities.contains(&capability.to_string())
    }

    /// 添加能力
    pub fn add_capability(&mut self, capability: String) {
        if !self.capabilities.contains(&capability) {
            self.capabilities.push(capability);
        }
    }

    /// 移除能力
    pub fn remove_capability(&mut self, capability: &str) {
        self.capabilities.retain(|c| c != capability);
    }

    /// 添加特征
    pub fn add_trait(&mut self, trait_: RoleTrait) {
        self.traits.push(trait_);
    }

    /// 移除特征
    pub fn remove_trait(&mut self, name: &str) -> Option<RoleTrait> {
        if let Some(index) = self.traits.iter().position(|t| t.name == name) {
            Some(self.traits.remove(index))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_creation() {
        let config = RoleConfig {
            name: "Assistant".to_string(),
            description: "A helpful assistant".to_string(),
            capabilities: vec!["chat".to_string(), "help".to_string()],
        };

        let role = Role::from_config(&config).unwrap();
        assert_eq!(role.name(), "Assistant");
        assert!(role.has_capability("chat"));
        assert!(role.has_capability("help"));
        assert!(!role.has_capability("admin"));
    }

    #[test]
    fn test_role_capabilities() {
        let config = RoleConfig {
            name: "Test".to_string(),
            description: "Test".to_string(),
            capabilities: vec!["cap1".to_string()],
        };

        let mut role = Role::from_config(&config).unwrap();
        assert!(role.has_capability("cap1"));

        role.add_capability("cap2".to_string());
        assert!(role.has_capability("cap2"));

        role.remove_capability("cap1");
        assert!(!role.has_capability("cap1"));
    }
}
