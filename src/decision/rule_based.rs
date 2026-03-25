//! 规则驱动决策引擎
//!
//! 基于预定义规则的决策引擎。

use super::engine::{Decision, DecisionContext, DecisionEngineTrait, DecisionEngineType};
use crate::error::FrameworkError;

/// 规则
pub struct Rule {
    /// 规则名称
    pub name: String,
    /// 条件函数
    pub condition: Box<dyn Fn(&DecisionContext) -> bool + Send + Sync>,
    /// 决策函数
    pub action: Box<dyn Fn(&DecisionContext) -> Decision + Send + Sync>,
    /// 优先级
    pub priority: i32,
}

/// 规则驱动引擎
pub struct RuleBasedEngine {
    /// 规则列表
    rules: Vec<Rule>,
    /// 默认决策
    default_decision: Option<Decision>,
}

impl RuleBasedEngine {
    /// 创建新的规则驱动引擎
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            default_decision: None,
        }
    }

    /// 添加规则
    pub fn add_rule(&mut self, rule: Rule) {
        self.rules.push(rule);
        // 按优先级排序
        self.rules.sort_by_key(|r| r.priority);
    }

    /// 设置默认决策
    pub fn set_default_decision(&mut self, decision: Decision) {
        self.default_decision = Some(decision);
    }
}

impl DecisionEngineTrait for RuleBasedEngine {
    fn decide(&self, context: &DecisionContext) -> Result<Decision, FrameworkError> {
        for rule in &self.rules {
            if (rule.condition)(context) {
                return Ok((rule.action)(context));
            }
        }

        // 如果没有匹配的规则，返回默认决策
        self.default_decision
            .clone()
            .ok_or_else(|| FrameworkError::Other("No matching rule found".to_string()))
    }

    fn name(&self) -> &str {
        "RuleBasedEngine"
    }

    fn engine_type(&self) -> DecisionEngineType {
        DecisionEngineType::RuleBased
    }
}

impl Default for RuleBasedEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::super::engine::DecisionType;
    use super::*;

    #[test]
    fn test_rule_based_engine() {
        let mut engine = RuleBasedEngine::new();

        // 添加一个简单的规则
        engine.add_rule(Rule {
            name: "greeting".to_string(),
            condition: Box::new(|ctx| ctx.input.contains("hello")),
            action: Box::new(|_| Decision {
                decision_type: DecisionType::Reply,
                content: serde_json::json!({"message": "Hello! How can I help you?"}),
                confidence: 1.0,
                reason: Some("Greeting detected".to_string()),
            }),
            priority: 100,
        });

        let context = DecisionContext {
            input: "hello there".to_string(),
            history: vec![],
            available_tools: vec![],
            agent_state: serde_json::json!({}),
        };

        let decision = engine.decide(&context).unwrap();
        assert!(matches!(decision.decision_type, DecisionType::Reply));
        assert_eq!(decision.confidence, 1.0);
    }
}
