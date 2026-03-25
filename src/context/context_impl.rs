//! 上下文模块
//!
//! 定义 Agent 的上下文信息。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Context {
    /// 会话 ID
    pub session_id: Option<String>,
    /// Agent ID
    pub agent_id: Option<String>,
    /// 输入消息
    pub input: Option<String>,
    /// 输出消息
    pub output: Option<String>,
    /// 历史记录
    pub history: Vec<HistoryEntry>,
    /// 上下文数据
    pub data: HashMap<String, serde_json::Value>,
    /// 元数据
    pub metadata: HashMap<String, String>,
}

/// 历史记录条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// 角色 (user/assistant/system)
    pub role: String,
    /// 内容
    pub content: String,
    /// 时间戳
    pub timestamp: u64,
}

impl Context {
    /// 创建新的上下文
    pub fn new() -> Self {
        Self {
            session_id: None,
            agent_id: None,
            input: None,
            output: None,
            history: Vec::new(),
            data: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    /// 设置会话 ID
    pub fn with_session_id(mut self, session_id: String) -> Self {
        self.session_id = Some(session_id);
        self
    }

    /// 设置 Agent ID
    pub fn with_agent_id(mut self, agent_id: String) -> Self {
        self.agent_id = Some(agent_id);
        self
    }

    /// 设置输入
    pub fn with_input(mut self, input: String) -> Self {
        self.input = Some(input);
        self
    }

    /// 设置输出
    pub fn with_output(mut self, output: String) -> Self {
        self.output = Some(output);
        self
    }

    /// 添加历史记录
    pub fn add_history(&mut self, role: String, content: String) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.history.push(HistoryEntry {
            role,
            content,
            timestamp,
        });
    }

    /// 获取数据
    pub fn get_data(&self, key: &str) -> Option<&serde_json::Value> {
        self.data.get(key)
    }

    /// 设置数据
    pub fn set_data(&mut self, key: String, value: serde_json::Value) {
        self.data.insert(key, value);
    }

    /// 获取元数据
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }

    /// 设置元数据
    pub fn set_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    /// 清空历史记录
    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    /// 获取历史记录数量
    pub fn history_len(&self) -> usize {
        self.history.len()
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_creation() {
        let context = Context::new();
        assert!(context.session_id.is_none());
        assert!(context.history.is_empty());
    }

    #[test]
    fn test_context_builder() {
        let context = Context::new()
            .with_session_id("session-1".to_string())
            .with_agent_id("agent-1".to_string())
            .with_input("hello".to_string());

        assert_eq!(context.session_id, Some("session-1".to_string()));
        assert_eq!(context.agent_id, Some("agent-1".to_string()));
        assert_eq!(context.input, Some("hello".to_string()));
    }

    #[test]
    fn test_context_history() {
        let mut context = Context::new();
        context.add_history("user".to_string(), "hello".to_string());
        context.add_history("assistant".to_string(), "hi there".to_string());

        assert_eq!(context.history_len(), 2);
    }
}
