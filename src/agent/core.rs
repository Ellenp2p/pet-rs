//! Agent 核心结构
//!
//! 定义 Agent 的核心数据结构和基本操作。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::context::context_impl::Context;
use crate::error::FrameworkError;
use crate::hooks::HookRegistry;
use crate::memory::memory_impl::Memory;

use super::personality::Personality;
use super::role::Role;

/// Agent 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Agent 名称
    pub name: String,
    /// Agent 描述
    pub description: String,
    /// 初始人格
    pub personality: PersonalityConfig,
    /// 初始角色
    pub role: RoleConfig,
    /// 记忆配置
    pub memory: MemoryConfig,
    /// 决策引擎配置
    pub decision: DecisionConfig,
}

/// 人格配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalityConfig {
    /// 人格名称
    pub name: String,
    /// 人格描述
    pub description: String,
    /// 人格特征
    pub traits: Vec<String>,
    /// 对话风格
    pub dialogue_style: String,
}

/// 角色配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleConfig {
    /// 角色名称
    pub name: String,
    /// 角色描述
    pub description: String,
    /// 角色能力
    pub capabilities: Vec<String>,
}

/// 记忆配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    /// 短期记忆容量
    pub short_term_capacity: usize,
    /// 长期记忆是否启用
    pub long_term_enabled: bool,
    /// 工作记忆容量
    pub working_capacity: usize,
}

/// 决策引擎配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionConfig {
    /// 决策引擎类型
    pub engine_type: DecisionEngineType,
    /// LLM 提供商（如果使用 LLM）
    pub llm_provider: Option<String>,
    /// LLM 模型（如果使用 LLM）
    pub llm_model: Option<String>,
}

/// 决策引擎类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DecisionEngineType {
    /// 规则驱动
    RuleBased,
    /// LLM 驱动
    LLM,
    /// 混合模式
    Hybrid,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            name: "Agent".to_string(),
            description: "A virtual agent".to_string(),
            personality: PersonalityConfig {
                name: "Friendly".to_string(),
                description: "A friendly and helpful agent".to_string(),
                traits: vec!["friendly".to_string(), "helpful".to_string()],
                dialogue_style: "casual".to_string(),
            },
            role: RoleConfig {
                name: "Assistant".to_string(),
                description: "A helpful assistant".to_string(),
                capabilities: vec!["chat".to_string(), "help".to_string()],
            },
            memory: MemoryConfig {
                short_term_capacity: 100,
                long_term_enabled: true,
                working_capacity: 10,
            },
            decision: DecisionConfig {
                engine_type: DecisionEngineType::RuleBased,
                llm_provider: None,
                llm_model: None,
            },
        }
    }
}

/// Agent 状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentState {
    /// 空闲
    Idle,
    /// 处理中
    Processing,
    /// 思考中
    Thinking,
    /// 执行动作
    Executing,
    /// 错误状态
    Error(String),
}

/// Agent 核心结构
pub struct Agent {
    /// Agent 配置
    config: AgentConfig,
    /// Agent 状态
    state: AgentState,
    /// Agent ID
    id: String,
    /// 当前人格
    personality: Personality,
    /// 当前角色
    role: Role,
    /// Hook 注册表
    hooks: HookRegistry,
    /// 记忆系统
    memory: Memory,
    /// 上下文
    context: Context,
    /// 自定义属性
    attributes: HashMap<String, serde_json::Value>,
}

impl Agent {
    /// 创建新的 Agent
    pub fn new(config: AgentConfig) -> Result<Self, FrameworkError> {
        let id = uuid::Uuid::new_v4().to_string();
        let personality = Personality::from_config(&config.personality)?;
        let role = Role::from_config(&config.role)?;
        let hooks = HookRegistry::default();
        let memory = Memory::new(&config.memory)?;
        let context = Context::new();

        Ok(Self {
            config,
            state: AgentState::Idle,
            id,
            personality,
            role,
            hooks,
            memory,
            context,
            attributes: HashMap::new(),
        })
    }

    /// 获取 Agent ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// 获取 Agent 名称
    pub fn name(&self) -> &str {
        &self.config.name
    }

    /// 获取 Agent 状态
    pub fn state(&self) -> &AgentState {
        &self.state
    }

    /// 设置 Agent 状态
    pub fn set_state(&mut self, state: AgentState) {
        self.state = state;
    }

    /// 获取 Agent 配置
    pub fn config(&self) -> &AgentConfig {
        &self.config
    }

    /// 获取当前人格
    pub fn personality(&self) -> &Personality {
        &self.personality
    }

    /// 获取当前角色
    pub fn role(&self) -> &Role {
        &self.role
    }

    /// 获取 Hook 注册表
    pub fn hooks(&self) -> &HookRegistry {
        &self.hooks
    }

    /// 获取可变 Hook 注册表
    pub fn hooks_mut(&mut self) -> &mut HookRegistry {
        &mut self.hooks
    }

    /// 获取记忆系统
    pub fn memory(&self) -> &Memory {
        &self.memory
    }

    /// 获取可变记忆系统
    pub fn memory_mut(&mut self) -> &mut Memory {
        &mut self.memory
    }

    /// 获取上下文
    pub fn context(&self) -> &Context {
        &self.context
    }

    /// 获取可变上下文
    pub fn context_mut(&mut self) -> &mut Context {
        &mut self.context
    }

    /// 获取自定义属性
    pub fn attribute(&self, key: &str) -> Option<&serde_json::Value> {
        self.attributes.get(key)
    }

    /// 设置自定义属性
    pub fn set_attribute(&mut self, key: String, value: serde_json::Value) {
        self.attributes.insert(key, value);
    }

    /// 切换人格
    pub fn switch_personality(&mut self, personality: Personality) -> Result<(), FrameworkError> {
        use crate::hooks::HookContext;
        use crate::hooks::HookPoint;

        // 触发 before_role_apply hook
        let mut context = HookContext::new(HookPoint::BeforeRoleApply, self.id.clone());
        context.set_data(
            "old_personality".to_string(),
            serde_json::json!(self.personality.name()),
        );
        context.set_data(
            "new_personality".to_string(),
            serde_json::json!(personality.name()),
        );
        self.hooks.trigger("before_role_apply", &context)?;

        // 切换人格
        let old_personality = std::mem::replace(&mut self.personality, personality);

        // 触发 after_role_apply hook
        let mut context = HookContext::new(HookPoint::AfterRoleApply, self.id.clone());
        context.set_data(
            "old_personality".to_string(),
            serde_json::json!(old_personality.name()),
        );
        context.set_data(
            "new_personality".to_string(),
            serde_json::json!(self.personality.name()),
        );
        self.hooks.trigger("after_role_apply", &context)?;

        // 触发 on_personality_change hook
        let mut context = HookContext::new(HookPoint::OnPersonalityChange, self.id.clone());
        context.set_data(
            "old_personality".to_string(),
            serde_json::json!(old_personality.name()),
        );
        context.set_data(
            "new_personality".to_string(),
            serde_json::json!(self.personality.name()),
        );
        self.hooks.trigger("on_personality_change", &context)?;

        Ok(())
    }

    /// 切换角色
    pub fn switch_role(&mut self, role: Role) -> Result<(), FrameworkError> {
        use crate::hooks::HookContext;
        use crate::hooks::HookPoint;

        // 触发 before_role_apply hook
        let mut context = HookContext::new(HookPoint::BeforeRoleApply, self.id.clone());
        context.set_data("old_role".to_string(), serde_json::json!(self.role.name()));
        context.set_data("new_role".to_string(), serde_json::json!(role.name()));
        self.hooks.trigger("before_role_apply", &context)?;

        // 切换角色
        let old_role = std::mem::replace(&mut self.role, role);

        // 触发 after_role_apply hook
        let mut context = HookContext::new(HookPoint::AfterRoleApply, self.id.clone());
        context.set_data("old_role".to_string(), serde_json::json!(old_role.name()));
        context.set_data("new_role".to_string(), serde_json::json!(self.role.name()));
        self.hooks.trigger("after_role_apply", &context)?;

        Ok(())
    }

    /// 启动 Agent
    pub fn start(&mut self) -> Result<(), FrameworkError> {
        use crate::hooks::HookContext;
        use crate::hooks::HookPoint;

        self.set_state(AgentState::Idle);
        let context = HookContext::new(HookPoint::OnAgentStart, self.id.clone());
        self.hooks.trigger("on_agent_start", &context)?;
        Ok(())
    }

    /// 停止 Agent
    pub fn stop(&mut self) -> Result<(), FrameworkError> {
        use crate::hooks::HookContext;
        use crate::hooks::HookPoint;

        let context = HookContext::new(HookPoint::OnAgentStop, self.id.clone());
        self.hooks.trigger("on_agent_stop", &context)?;
        self.set_state(AgentState::Idle);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_creation() {
        let config = AgentConfig::default();
        let agent = Agent::new(config).unwrap();
        assert_eq!(agent.name(), "Agent");
        assert!(matches!(agent.state(), AgentState::Idle));
    }

    #[test]
    fn test_agent_attributes() {
        let config = AgentConfig::default();
        let mut agent = Agent::new(config).unwrap();

        agent.set_attribute("mood".to_string(), serde_json::json!("happy"));
        assert_eq!(agent.attribute("mood"), Some(&serde_json::json!("happy")));
    }

    #[test]
    fn test_agent_lifecycle() {
        let config = AgentConfig::default();
        let mut agent = Agent::new(config).unwrap();

        assert!(agent.start().is_ok());
        assert!(agent.stop().is_ok());
    }
}
