//! 决策引擎模块
//!
//! 提供规则驱动、LLM 驱动和混合模式的决策引擎。

pub mod engine;
pub mod hybrid;
pub mod llm_based;
pub mod rule_based;

pub use engine::{
    Decision, DecisionContext, DecisionEngine, DecisionEngineTrait, DecisionEngineType,
    DecisionType,
};
pub use hybrid::{HybridConfig, HybridEngine, HybridStrategy};
pub use llm_based::{LLMConfig, LLMEngine, PromptTemplate};
pub use rule_based::{Rule, RuleBasedEngine};
