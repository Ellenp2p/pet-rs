//! 混合决策引擎
//!
//! 结合规则引擎和 LLM 引擎的混合决策系统。

use super::engine::{Decision, DecisionContext, DecisionEngineTrait, DecisionEngineType};
use super::llm_based::{LLMConfig, LLMEngine};
use super::rule_based::RuleBasedEngine;
use crate::error::FrameworkError;
use serde::{Deserialize, Serialize};

/// 混合策略
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum HybridStrategy {
    /// 规则优先：先尝试规则，失败时使用 LLM
    #[default]
    RuleFirst,
    /// LLM 优先：先尝试 LLM，失败时使用规则
    LLMFirst,
    /// 并行：同时运行规则和 LLM，选择置信度更高的
    Parallel,
    /// 顺序：先规则，然后用 LLM 增强
    Sequential,
}

/// 混合引擎配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridConfig {
    /// 混合策略
    pub strategy: HybridStrategy,
    /// LLM 配置
    pub llm_config: LLMConfig,
    /// 是否启用 LLM
    pub llm_enabled: bool,
    /// LLM 置信度阈值
    pub llm_confidence_threshold: f32,
}

impl Default for HybridConfig {
    fn default() -> Self {
        Self {
            strategy: HybridStrategy::RuleFirst,
            llm_config: LLMConfig::default(),
            llm_enabled: true,
            llm_confidence_threshold: 0.7,
        }
    }
}

/// 混合决策引擎
pub struct HybridEngine {
    /// 规则引擎
    rule_engine: RuleBasedEngine,
    /// LLM 引擎
    llm_engine: LLMEngine,
    /// 配置
    config: HybridConfig,
}

impl HybridEngine {
    /// 创建新的混合引擎
    pub fn new(rule_engine: RuleBasedEngine, llm_engine: LLMEngine, config: HybridConfig) -> Self {
        Self {
            rule_engine,
            llm_engine,
            config,
        }
    }

    /// 使用默认配置创建混合引擎
    pub fn with_default_config() -> Result<Self, FrameworkError> {
        let rule_engine = RuleBasedEngine::new();
        let llm_config = LLMConfig::default();
        let llm_engine = LLMEngine::new(llm_config);
        let config = HybridConfig::default();

        Ok(Self::new(rule_engine, llm_engine, config))
    }

    /// 设置混合策略
    pub fn with_strategy(mut self, strategy: HybridStrategy) -> Self {
        self.config.strategy = strategy;
        self
    }

    /// 启用/禁用 LLM
    pub fn with_llm_enabled(mut self, enabled: bool) -> Self {
        self.config.llm_enabled = enabled;
        self
    }

    /// 添加规则
    pub fn add_rule(&mut self, rule: super::rule_based::Rule) {
        self.rule_engine.add_rule(rule);
    }

    /// 规则优先策略
    fn decide_rule_first(&self, context: &DecisionContext) -> Result<Decision, FrameworkError> {
        // 先尝试规则引擎
        match self.rule_engine.decide(context) {
            Ok(decision) => {
                if decision.confidence >= self.config.llm_confidence_threshold {
                    return Ok(decision);
                }
            }
            Err(_) => {
                // 规则引擎没有匹配的规则
            }
        }

        // 如果规则引擎失败或置信度低，使用 LLM
        if self.config.llm_enabled {
            self.llm_engine.decide(context)
        } else {
            self.rule_engine.decide(context)
        }
    }

    /// LLM 优先策略
    fn decide_llm_first(&self, context: &DecisionContext) -> Result<Decision, FrameworkError> {
        if self.config.llm_enabled {
            match self.llm_engine.decide(context) {
                Ok(decision) => {
                    if decision.confidence >= self.config.llm_confidence_threshold {
                        return Ok(decision);
                    }
                }
                Err(_) => {
                    // LLM 引擎失败
                }
            }
        }

        // 如果 LLM 失败或置信度低，使用规则引擎
        self.rule_engine.decide(context)
    }

    /// 并行策略
    fn decide_parallel(&self, context: &DecisionContext) -> Result<Decision, FrameworkError> {
        let rule_result = self.rule_engine.decide(context).ok();
        let llm_result = if self.config.llm_enabled {
            self.llm_engine.decide(context).ok()
        } else {
            None
        };

        // 比较置信度，选择更高的
        match (rule_result, llm_result) {
            (Some(rule), Some(llm)) => {
                if rule.confidence >= llm.confidence {
                    Ok(rule)
                } else {
                    Ok(llm)
                }
            }
            (Some(rule), None) => Ok(rule),
            (None, Some(llm)) => Ok(llm),
            (None, None) => Err(FrameworkError::Other(
                "Both rule and LLM engines failed".to_string(),
            )),
        }
    }

    /// 顺序策略
    fn decide_sequential(&self, context: &DecisionContext) -> Result<Decision, FrameworkError> {
        // 先运行规则引擎
        let rule_decision = self.rule_engine.decide(context)?;

        // 如果 LLM 启用，用 LLM 增强决策
        if self.config.llm_enabled {
            match self.llm_engine.decide(context) {
                Ok(llm_decision) => {
                    // 合并规则和 LLM 的决策
                    // 这里可以根据需要实现更复杂的合并逻辑
                    if llm_decision.confidence > rule_decision.confidence {
                        Ok(llm_decision)
                    } else {
                        Ok(rule_decision)
                    }
                }
                Err(_) => Ok(rule_decision),
            }
        } else {
            Ok(rule_decision)
        }
    }
}

impl DecisionEngineTrait for HybridEngine {
    fn decide(&self, context: &DecisionContext) -> Result<Decision, FrameworkError> {
        match self.config.strategy {
            HybridStrategy::RuleFirst => self.decide_rule_first(context),
            HybridStrategy::LLMFirst => self.decide_llm_first(context),
            HybridStrategy::Parallel => self.decide_parallel(context),
            HybridStrategy::Sequential => self.decide_sequential(context),
        }
    }

    fn name(&self) -> &str {
        "HybridEngine"
    }

    fn engine_type(&self) -> DecisionEngineType {
        DecisionEngineType::Hybrid
    }
}

#[cfg(test)]
mod tests {
    use super::super::rule_based::Rule;
    use super::*;

    #[test]
    fn test_hybrid_engine_creation() {
        let engine = HybridEngine::with_default_config().unwrap();
        assert_eq!(engine.name(), "HybridEngine");
        assert!(matches!(engine.engine_type(), DecisionEngineType::Hybrid));
    }

    #[test]
    fn test_hybrid_engine_rule_first() {
        let rule_engine = RuleBasedEngine::new();
        let llm_engine = LLMEngine::new(LLMConfig::default());
        let config = HybridConfig {
            strategy: HybridStrategy::RuleFirst,
            ..Default::default()
        };

        let mut engine = HybridEngine::new(rule_engine, llm_engine, config);

        // 添加一个规则
        engine.add_rule(Rule {
            name: "greeting".to_string(),
            condition: Box::new(|ctx| ctx.input.contains("hello")),
            action: Box::new(|_| Decision {
                decision_type: super::super::engine::DecisionType::Reply,
                content: serde_json::json!({"message": "Hello!"}),
                confidence: 0.9,
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
        assert!(matches!(
            decision.decision_type,
            super::super::engine::DecisionType::Reply
        ));
    }
}
