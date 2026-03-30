//! 统一消息格式

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 会话 ID
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct SessionId {
    /// 渠道名称 (telegram, discord, cli, etc.)
    pub channel: String,
    /// 平台特定的聊天 ID
    pub chat_id: String,
}

impl SessionId {
    pub fn new(channel: &str, chat_id: &str) -> Self {
        Self {
            channel: channel.to_string(),
            chat_id: chat_id.to_string(),
        }
    }

    pub fn as_key(&self) -> String {
        format!("{}:{}", self.channel, self.chat_id)
    }
}

impl std::fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.channel, self.chat_id)
    }
}

/// 发送者信息
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SenderInfo {
    /// 发送者 ID (平台特定)
    pub id: String,
    /// 显示名称
    pub name: Option<String>,
    /// 是否是机器人
    pub is_bot: bool,
}

impl Default for SenderInfo {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: None,
            is_bot: false,
        }
    }
}

/// 媒体类型
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MediaType {
    Photo,
    Video,
    Audio,
    Document,
    Sticker,
    Voice,
    VideoNote,
}

/// 消息内容
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MessageContent {
    /// 纯文本
    Text(String),
    /// Markdown 格式
    Markdown(String),
    /// HTML 格式
    Html(String),
    /// 命令
    Command { command: String, args: Vec<String> },
    /// 媒体
    Media {
        media_type: MediaType,
        url: String,
        caption: Option<String>,
    },
    /// 位置
    Location { latitude: f64, longitude: f64 },
    /// 回复消息
    Reply {
        reply_to_id: String,
        content: Box<MessageContent>,
    },
}

impl MessageContent {
    /// 提取文本内容
    pub fn text(&self) -> Option<&str> {
        match self {
            MessageContent::Text(t) => Some(t),
            MessageContent::Markdown(t) => Some(t),
            MessageContent::Html(t) => Some(t),
            MessageContent::Reply { content, .. } => content.text(),
            _ => None,
        }
    }

    /// 是否是命令
    pub fn is_command(&self) -> bool {
        matches!(self, MessageContent::Command { .. })
    }
}

/// 入站消息 (从平台接收)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChannelMessage {
    /// 消息 ID (平台特定)
    pub id: String,
    /// 会话 ID
    pub session_id: SessionId,
    /// 发送者
    pub sender: SenderInfo,
    /// 消息内容
    pub content: MessageContent,
    /// 时间戳 (Unix 秒)
    pub timestamp: u64,
    /// 元数据 (平台特定)
    pub metadata: HashMap<String, String>,
}

impl ChannelMessage {
    /// 创建新消息
    pub fn new(
        id: String,
        session_id: SessionId,
        sender: SenderInfo,
        content: MessageContent,
    ) -> Self {
        Self {
            id,
            session_id,
            sender,
            content,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            metadata: HashMap::new(),
        }
    }

    /// 添加元数据
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
}

/// 出站消息 (发送到平台)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OutboundMessage {
    /// 会话 ID
    pub session_id: SessionId,
    /// 消息内容
    pub content: MessageContent,
    /// 回复的消息 ID (可选)
    pub reply_to: Option<String>,
    /// 元数据
    pub metadata: HashMap<String, String>,
}

impl OutboundMessage {
    /// 创建新的出站消息
    pub fn new(session_id: SessionId, content: MessageContent) -> Self {
        Self {
            session_id,
            content,
            reply_to: None,
            metadata: HashMap::new(),
        }
    }

    /// 设置回复目标
    pub fn reply_to(mut self, message_id: &str) -> Self {
        self.reply_to = Some(message_id.to_string());
        self
    }

    /// 添加元数据
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
}
