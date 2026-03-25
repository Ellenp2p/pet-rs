//! 上下文构建器
//!
//! 用于构建 Agent 的上下文。

use super::context_impl::Context;
use crate::error::FrameworkError;

/// 上下文构建器
pub struct ContextBuilder {
    /// 上下文
    context: Context,
}

impl ContextBuilder {
    /// 创建新的上下文构建器
    pub fn new() -> Self {
        Self {
            context: Context::new(),
        }
    }

    /// 设置会话 ID
    pub fn with_session_id(mut self, session_id: String) -> Self {
        self.context.session_id = Some(session_id);
        self
    }

    /// 设置 Agent ID
    pub fn with_agent_id(mut self, agent_id: String) -> Self {
        self.context.agent_id = Some(agent_id);
        self
    }

    /// 设置输入
    pub fn with_input(mut self, input: String) -> Self {
        self.context.input = Some(input);
        self
    }

    /// 添加数据
    pub fn with_data(mut self, key: String, value: serde_json::Value) -> Self {
        self.context.data.insert(key, value);
        self
    }

    /// 添加元数据
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.context.metadata.insert(key, value);
        self
    }

    /// 构建上下文
    pub fn build(self) -> Result<Context, FrameworkError> {
        Ok(self.context)
    }
}

impl Default for ContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_builder() {
        let context = ContextBuilder::new()
            .with_session_id("session-1".to_string())
            .with_agent_id("agent-1".to_string())
            .with_input("hello".to_string())
            .build()
            .unwrap();

        assert_eq!(context.session_id, Some("session-1".to_string()));
        assert_eq!(context.agent_id, Some("agent-1".to_string()));
        assert_eq!(context.input, Some("hello".to_string()));
    }
}
