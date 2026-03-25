//! Agent 人格系统
//!
//! 定义 Agent 的人格特征和对话风格。

use crate::agent::core::PersonalityConfig;
use crate::error::FrameworkError;
use serde::{Deserialize, Serialize};

/// 人格特征
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalityTrait {
    /// 特征名称
    pub name: String,
    /// 特征描述
    pub description: String,
    /// 特征值 (0.0 - 1.0)
    pub value: f32,
}

/// 人格
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Personality {
    /// 人格名称
    name: String,
    /// 人格描述
    description: String,
    /// 人格特征
    traits: Vec<PersonalityTrait>,
    /// 对话风格
    dialogue_style: String,
}

impl Personality {
    /// 从配置创建人格
    pub fn from_config(config: &PersonalityConfig) -> Result<Self, FrameworkError> {
        let traits = config
            .traits
            .iter()
            .map(|name| PersonalityTrait {
                name: name.clone(),
                description: format!("Trait: {}", name),
                value: 0.5,
            })
            .collect();

        Ok(Self {
            name: config.name.clone(),
            description: config.description.clone(),
            traits,
            dialogue_style: config.dialogue_style.clone(),
        })
    }

    /// 获取人格名称
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 获取人格描述
    pub fn description(&self) -> &str {
        &self.description
    }

    /// 获取人格特征
    pub fn traits(&self) -> &[PersonalityTrait] {
        &self.traits
    }

    /// 获取对话风格
    pub fn dialogue_style(&self) -> &str {
        &self.dialogue_style
    }

    /// 获取特征值
    pub fn get_trait(&self, name: &str) -> Option<&PersonalityTrait> {
        self.traits.iter().find(|t| t.name == name)
    }

    /// 设置特征值
    pub fn set_trait(&mut self, name: &str, value: f32) -> Result<(), FrameworkError> {
        if let Some(trait_) = self.traits.iter_mut().find(|t| t.name == name) {
            trait_.value = value.clamp(0.0, 1.0);
            Ok(())
        } else {
            Err(FrameworkError::Other(format!("Trait not found: {}", name)))
        }
    }

    /// 添加特征
    pub fn add_trait(&mut self, trait_: PersonalityTrait) {
        self.traits.push(trait_);
    }

    /// 移除特征
    pub fn remove_trait(&mut self, name: &str) -> Option<PersonalityTrait> {
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
    fn test_personality_creation() {
        let config = PersonalityConfig {
            name: "Friendly".to_string(),
            description: "A friendly agent".to_string(),
            traits: vec!["friendly".to_string(), "helpful".to_string()],
            dialogue_style: "casual".to_string(),
        };

        let personality = Personality::from_config(&config).unwrap();
        assert_eq!(personality.name(), "Friendly");
        assert_eq!(personality.traits().len(), 2);
    }

    #[test]
    fn test_personality_traits() {
        let config = PersonalityConfig {
            name: "Test".to_string(),
            description: "Test".to_string(),
            traits: vec!["trait1".to_string()],
            dialogue_style: "test".to_string(),
        };

        let mut personality = Personality::from_config(&config).unwrap();
        assert!(personality.get_trait("trait1").is_some());

        personality.set_trait("trait1", 0.8).unwrap();
        assert_eq!(personality.get_trait("trait1").unwrap().value, 0.8);
    }
}
