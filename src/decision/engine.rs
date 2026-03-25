//! 决策引擎 trait
//!
//! 定义决策引擎的通用接口。

use crate::error::FrameworkError;
use serde::{Deserialize, Serialize};

use super::rule_based::RuleBasedEngine;

/// 决策
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    /// 决策类型
    pub decision_type: DecisionType,
    /// 决策内容
    pub content: serde_json::Value,
    /// 置信度 (0.0 - 1.0)
    pub confidence: f32,
    /// 决策原因
    pub reason: Option<String>,
}

/// 决策类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DecisionType {
    /// 回复消息
    Reply,
    /// 执行动作
    Action,
    /// 调用工具
    ToolCall,
    /// 请求更多信息
    RequestInfo,
    /// 结束会话
    EndSession,
    /// 自定义
    Custom(String),
}

/// 决策上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionContext {
    /// 输入消息
    pub input: String,
    /// 历史记录
    pub history: Vec<String>,
    /// 可用工具
    pub available_tools: Vec<String>,
    /// Agent 状态
    pub agent_state: serde_json::Value,
}

/// 决策引擎 trait
pub trait DecisionEngineTrait {
    /// 做出决策
    fn decide(&self, context: &DecisionContext) -> Result<Decision, FrameworkError>;

    /// 获取引擎名称
    fn name(&self) -> &str;

    /// 获取引擎类型
    fn engine_type(&self) -> DecisionEngineType;
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

/// 决策引擎（枚举包装）
pub enum DecisionEngine {
    /// 规则驱动引擎
    RuleBased(RuleBasedEngine),
    // LLM(LLMEngine),
    // Hybrid(HybridEngine),
}

impl DecisionEngineTrait for DecisionEngine {
    fn decide(&self, context: &DecisionContext) -> Result<Decision, FrameworkError> {
        match self {
            DecisionEngine::RuleBased(engine) => engine.decide(context),
            // DecisionEngine::LLM(engine) => engine.decide(context),
            // DecisionEngine::Hybrid(engine) => engine.decide(context),
        }
    }

    fn name(&self) -> &str {
        match self {
            DecisionEngine::RuleBased(engine) => engine.name(),
            // DecisionEngine::LLM(engine) => engine.name(),
            // DecisionEngine::Hybrid(engine) => engine.name(),
        }
    }

    fn engine_type(&self) -> DecisionEngineType {
        match self {
            DecisionEngine::RuleBased(engine) => engine.engine_type(),
            // DecisionEngine::LLM(engine) => engine.engine_type(),
            // DecisionEngine::Hybrid(engine) => engine.engine_type(),
        }
    }
}
