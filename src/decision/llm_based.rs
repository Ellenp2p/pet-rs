//! LLM 驱动决策引擎
//!
//! 使用大语言模型进行决策的引擎。

use super::engine::{
    Decision, DecisionContext, DecisionEngineTrait, DecisionEngineType, DecisionType,
};
use crate::error::FrameworkError;
use serde::{Deserialize, Serialize};

/// LLM 提供商配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfig {
    /// 提供商名称
    pub provider: String,
    /// 模型名称
    pub model: String,
    /// API 基础 URL
    pub api_base: Option<String>,
    /// API 密钥
    pub api_key: Option<String>,
    /// 最大 token 数
    pub max_tokens: Option<u32>,
    /// 温度
    pub temperature: Option<f32>,
}

impl Default for LLMConfig {
    fn default() -> Self {
        Self {
            provider: "openai".to_string(),
            model: "gpt-3.5-turbo".to_string(),
            api_base: None,
            api_key: None,
            max_tokens: Some(1000),
            temperature: Some(0.7),
        }
    }
}

/// Prompt 模板
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplate {
    /// 系统提示
    pub system_prompt: String,
    /// 用户提示模板
    pub user_template: String,
}

impl Default for PromptTemplate {
    fn default() -> Self {
        Self {
            system_prompt: "You are a helpful AI assistant that can perform various tasks.".to_string(),
            user_template: "Input: {input}\n\nPlease respond with a JSON object containing:\n- action: the action to take (reply/action/tool_call/request_info/end_session)\n- content: the content of the response\n- confidence: confidence level (0.0-1.0)".to_string(),
        }
    }
}

/// LLM 响应解析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMResponse {
    /// 动作类型
    pub action: String,
    /// 内容
    pub content: serde_json::Value,
    /// 置信度
    pub confidence: f32,
    /// 原因（可选）
    pub reason: Option<String>,
}

/// LLM 驱动引擎
pub struct LLMEngine {
    /// 配置
    config: LLMConfig,
    /// Prompt 模板
    prompt_template: PromptTemplate,
    /// HTTP 客户端
    #[cfg(feature = "wasm-plugin")]
    client: Option<reqwest::Client>,
}

impl LLMEngine {
    /// 创建新的 LLM 引擎
    pub fn new(config: LLMConfig) -> Self {
        Self {
            config,
            prompt_template: PromptTemplate::default(),
            #[cfg(feature = "wasm-plugin")]
            client: None,
        }
    }

    /// 设置 Prompt 模板
    pub fn with_prompt_template(mut self, template: PromptTemplate) -> Self {
        self.prompt_template = template;
        self
    }

    /// 构建 Prompt
    fn build_prompt(&self, context: &DecisionContext) -> String {
        self.prompt_template
            .user_template
            .replace("{input}", &context.input)
    }

    /// 解析 LLM 响应
    fn parse_response(&self, response: &str) -> Result<LLMResponse, FrameworkError> {
        // 尝试解析 JSON 响应
        let parsed: LLMResponse = serde_json::from_str(response)
            .map_err(|e| FrameworkError::Other(format!("Failed to parse LLM response: {}", e)))?;

        Ok(parsed)
    }

    /// 调用 LLM API（占位符实现）
    #[cfg(feature = "wasm-plugin")]
    async fn call_llm(&self, prompt: &str) -> Result<String, FrameworkError> {
        // 这里应该实际调用 LLM API
        // 目前返回一个模拟响应
        let _ = prompt;

        // 模拟响应
        let response = serde_json::json!({
            "action": "reply",
            "content": {"message": "I'm a simulated LLM response."},
            "confidence": 0.8,
            "reason": "Simulated response for testing"
        });

        Ok(response.to_string())
    }
}

impl DecisionEngineTrait for LLMEngine {
    fn decide(&self, context: &DecisionContext) -> Result<Decision, FrameworkError> {
        let prompt = self.build_prompt(context);

        // 在同步上下文中，我们需要使用 block_on 或返回一个模拟决策
        // 这里我们先返回一个占位符决策
        let _ = prompt;

        // 解析响应
        let response = r#"{
            "action": "reply",
            "content": {"message": "LLM decision placeholder"},
            "confidence": 0.7,
            "reason": "Placeholder implementation"
        }"#;

        let parsed = self.parse_response(response)?;

        // 转换为 Decision
        let decision_type = match parsed.action.as_str() {
            "reply" => DecisionType::Reply,
            "action" => DecisionType::Action,
            "tool_call" => DecisionType::ToolCall,
            "request_info" => DecisionType::RequestInfo,
            "end_session" => DecisionType::EndSession,
            _ => DecisionType::Custom(parsed.action),
        };

        Ok(Decision {
            decision_type,
            content: parsed.content,
            confidence: parsed.confidence,
            reason: parsed.reason,
        })
    }

    fn name(&self) -> &str {
        "LLMEngine"
    }

    fn engine_type(&self) -> DecisionEngineType {
        DecisionEngineType::LLM
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llm_engine_creation() {
        let config = LLMConfig::default();
        let engine = LLMEngine::new(config);
        assert_eq!(engine.name(), "LLMEngine");
        assert!(matches!(engine.engine_type(), DecisionEngineType::LLM));
    }

    #[test]
    fn test_llm_engine_decide() {
        let config = LLMConfig::default();
        let engine = LLMEngine::new(config);

        let context = DecisionContext {
            input: "Hello, how are you?".to_string(),
            history: vec![],
            available_tools: vec![],
            agent_state: serde_json::json!({}),
        };

        let decision = engine.decide(&context).unwrap();
        assert!(matches!(decision.decision_type, DecisionType::Reply));
        assert!(decision.confidence > 0.0);
    }
}
