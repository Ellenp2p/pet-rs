//! 消息模块
//!
//! 定义消息结构。

use serde::{Deserialize, Serialize};

/// 消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// 消息 ID
    pub id: String,
    /// 发送者
    pub sender: String,
    /// 内容
    pub content: String,
    /// 时间戳
    pub timestamp: u64,
    /// 消息类型
    pub message_type: MessageType,
    /// 元数据
    pub metadata: std::collections::HashMap<String, String>,
}

/// 消息类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    /// 文本消息
    Text,
    /// 命令消息
    Command,
    /// 事件消息
    Event,
    /// 系统消息
    System,
}

impl Message {
    /// 创建新的消息
    pub fn new(sender: String, content: String) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            sender,
            content,
            timestamp,
            message_type: MessageType::Text,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// 创建文本消息
    pub fn text(sender: String, content: String) -> Self {
        let mut message = Self::new(sender, content);
        message.message_type = MessageType::Text;
        message
    }

    /// 创建命令消息
    pub fn command(sender: String, content: String) -> Self {
        let mut message = Self::new(sender, content);
        message.message_type = MessageType::Command;
        message
    }

    /// 创建事件消息
    pub fn event(sender: String, content: String) -> Self {
        let mut message = Self::new(sender, content);
        message.message_type = MessageType::Event;
        message
    }

    /// 创建系统消息
    pub fn system(content: String) -> Self {
        let mut message = Self::new("system".to_string(), content);
        message.message_type = MessageType::System;
        message
    }

    /// 设置元数据
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let message = Message::new("user".to_string(), "hello".to_string());
        assert_eq!(message.sender, "user");
        assert_eq!(message.content, "hello");
    }

    #[test]
    fn test_message_types() {
        let text_msg = Message::text("user".to_string(), "hello".to_string());
        assert!(matches!(text_msg.message_type, MessageType::Text));

        let cmd_msg = Message::command("user".to_string(), "/help".to_string());
        assert!(matches!(cmd_msg.message_type, MessageType::Command));

        let sys_msg = Message::system("System starting".to_string());
        assert!(matches!(sys_msg.message_type, MessageType::System));
    }
}
