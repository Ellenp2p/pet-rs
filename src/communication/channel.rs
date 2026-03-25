//! 通道模块
//!
//! 定义通信通道接口。

use crate::error::FrameworkError;
use serde::{Deserialize, Serialize};

use super::message::Message;

/// 通道类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChannelType {
    /// CLI 通道
    CLI,
    /// HTTP 通道
    HTTP,
    /// WebSocket 通道
    WebSocket,
    /// 自定义通道
    Custom(String),
}

/// 通道 trait
pub trait ChannelTrait {
    /// 发送消息
    fn send(&self, message: Message) -> Result<(), FrameworkError>;

    /// 接收消息
    fn receive(&self) -> Result<Option<Message>, FrameworkError>;

    /// 获取通道类型
    fn channel_type(&self) -> ChannelType;

    /// 获取通道名称
    fn name(&self) -> &str;
}

/// 通道
pub enum Channel {
    /// CLI 通道
    CLI(CLIChannel),
}

/// CLI 通道
pub struct CLIChannel {
    /// 名称
    name: String,
}

impl CLIChannel {
    /// 创建新的 CLI 通道
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

impl ChannelTrait for CLIChannel {
    fn send(&self, message: Message) -> Result<(), FrameworkError> {
        println!("{}", message.content);
        Ok(())
    }

    fn receive(&self) -> Result<Option<Message>, FrameworkError> {
        use std::io::{self, BufRead};
        let stdin = io::stdin();
        let mut line = String::new();
        stdin
            .lock()
            .read_line(&mut line)
            .map_err(|e| FrameworkError::Other(format!("Failed to read from stdin: {}", e)))?;

        if line.is_empty() {
            Ok(None)
        } else {
            Ok(Some(Message::new(
                "user".to_string(),
                line.trim().to_string(),
            )))
        }
    }

    fn channel_type(&self) -> ChannelType {
        ChannelType::CLI
    }

    fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_channel() {
        let channel = CLIChannel::new("test".to_string());
        assert_eq!(channel.name(), "test");
        assert!(matches!(channel.channel_type(), ChannelType::CLI));
    }
}
