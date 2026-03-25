//! Hook 上下文
//!
//! 定义 Hook 执行时的上下文信息。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::points::HookPoint;

/// Hook 上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookContext {
    /// Hook 点
    pub hook_point: HookPoint,
    /// Agent ID
    pub agent_id: String,
    /// 会话 ID（如果有）
    pub session_id: Option<String>,
    /// 输入数据
    pub input: Option<serde_json::Value>,
    /// 输出数据
    pub output: Option<serde_json::Value>,
    /// 上下文数据
    pub data: HashMap<String, serde_json::Value>,
    /// 元数据
    pub metadata: HashMap<String, String>,
}

impl HookContext {
    /// 创建新的 Hook 上下文
    pub fn new(hook_point: HookPoint, agent_id: String) -> Self {
        Self {
            hook_point,
            agent_id,
            session_id: None,
            input: None,
            output: None,
            data: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    /// 设置会话 ID
    pub fn with_session_id(mut self, session_id: String) -> Self {
        self.session_id = Some(session_id);
        self
    }

    /// 设置输入数据
    pub fn with_input(mut self, input: serde_json::Value) -> Self {
        self.input = Some(input);
        self
    }

    /// 设置输出数据
    pub fn with_output(mut self, output: serde_json::Value) -> Self {
        self.output = Some(output);
        self
    }

    /// 设置上下文数据
    pub fn with_data(mut self, key: String, value: serde_json::Value) -> Self {
        self.data.insert(key, value);
        self
    }

    /// 设置元数据
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// 获取上下文数据
    pub fn get_data(&self, key: &str) -> Option<&serde_json::Value> {
        self.data.get(key)
    }

    /// 获取元数据
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }

    /// 设置上下文数据
    pub fn set_data(&mut self, key: String, value: serde_json::Value) {
        self.data.insert(key, value);
    }

    /// 设置元数据
    pub fn set_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
}

/// Hook 结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HookResult {
    /// 继续执行
    Continue,
    /// 修改数据后继续
    Modified(serde_json::Value),
    /// 阻止执行
    Blocked {
        /// 阻止原因
        reason: String,
    },
    /// 跳过（仅用于 before_tool_call）
    Skip,
    /// 替换为其他操作
    Replace(serde_json::Value),
}

impl HookResult {
    /// 是否应该继续执行
    pub fn should_continue(&self) -> bool {
        matches!(self, HookResult::Continue | HookResult::Modified(_))
    }

    /// 是否被阻止
    pub fn is_blocked(&self) -> bool {
        matches!(self, HookResult::Blocked { .. })
    }

    /// 是否应该跳过
    pub fn should_skip(&self) -> bool {
        matches!(self, HookResult::Skip)
    }

    /// 是否应该替换
    pub fn should_replace(&self) -> bool {
        matches!(self, HookResult::Replace(_))
    }

    /// 获取修改后的数据
    pub fn modified_data(&self) -> Option<&serde_json::Value> {
        match self {
            HookResult::Modified(data) => Some(data),
            HookResult::Replace(data) => Some(data),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_context_creation() {
        let context = HookContext::new(HookPoint::OnInputReceived, "agent-1".to_string());
        assert_eq!(context.hook_point, HookPoint::OnInputReceived);
        assert_eq!(context.agent_id, "agent-1");
    }

    #[test]
    fn test_hook_context_builder() {
        let context = HookContext::new(HookPoint::BeforeDecision, "agent-1".to_string())
            .with_session_id("session-1".to_string())
            .with_input(serde_json::json!({"message": "hello"}))
            .with_data("key".to_string(), serde_json::json!("value"));

        assert_eq!(context.session_id, Some("session-1".to_string()));
        assert!(context.input.is_some());
        assert_eq!(context.get_data("key"), Some(&serde_json::json!("value")));
    }

    #[test]
    fn test_hook_result() {
        let result = HookResult::Continue;
        assert!(result.should_continue());
        assert!(!result.is_blocked());

        let result = HookResult::Blocked {
            reason: "test".to_string(),
        };
        assert!(!result.should_continue());
        assert!(result.is_blocked());
    }
}
