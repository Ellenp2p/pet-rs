//! 渠道适配器接口

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;

use super::message::{ChannelMessage, OutboundMessage};

/// 渠道错误
#[derive(Debug, Error)]
pub enum ChannelError {
    #[error("连接失败: {0}")]
    ConnectionFailed(String),

    #[error("发送失败: {0}")]
    SendFailed(String),

    #[error("认证失败: {0}")]
    AuthFailed(String),

    #[error("配置错误: {0}")]
    ConfigError(String),

    #[error("序列化错误: {0}")]
    SerializeError(#[from] serde_json::Error),

    #[error("WASM 错误: {0}")]
    WasmError(String),

    #[error("渠道未连接")]
    NotConnected,

    #[error("渠道已连接")]
    AlreadyConnected,

    #[error("超时")]
    Timeout,

    #[error("未知错误: {0}")]
    Unknown(String),
}

/// 渠道类型
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChannelType {
    /// CLI 终端
    Cli,
    /// HTTP API
    Http,
    /// WebSocket
    WebSocket,
    /// 自定义渠道
    Custom(String),
}

impl std::fmt::Display for ChannelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChannelType::Cli => write!(f, "cli"),
            ChannelType::Http => write!(f, "http"),
            ChannelType::WebSocket => write!(f, "websocket"),
            ChannelType::Custom(name) => write!(f, "{}", name),
        }
    }
}

/// 渠道配置
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChannelConfig {
    /// 渠道名称
    pub name: String,
    /// 认证 Token
    pub auth_token: String,
    /// API 基础 URL (可选)
    pub api_base: Option<String>,
    /// 轮询间隔 (毫秒)
    pub poll_interval_ms: u64,
    /// 是否启用
    pub enabled: bool,
    /// 额外配置
    pub extra: HashMap<String, String>,
}

impl Default for ChannelConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            auth_token: String::new(),
            api_base: None,
            poll_interval_ms: 1000,
            enabled: true,
            extra: HashMap::new(),
        }
    }
}

/// 渠道状态
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChannelStatus {
    /// 是否已连接
    pub connected: bool,
    /// 待处理消息数
    pub pending_messages: u64,
    /// 最后活动时间
    pub last_activity: u64,
    /// 错误信息 (如果有)
    pub error: Option<String>,
}

impl Default for ChannelStatus {
    fn default() -> Self {
        Self {
            connected: false,
            pending_messages: 0,
            last_activity: 0,
            error: None,
        }
    }
}

/// 消息处理器
#[async_trait]
pub trait MessageHandler: Send + Sync {
    /// 处理入站消息
    async fn handle_message(
        &self,
        message: ChannelMessage,
    ) -> Result<Option<OutboundMessage>, ChannelError>;
}

/// 简单消息处理器 (使用闭包)
pub struct FnMessageHandler<F>
where
    F: Fn(ChannelMessage) -> Result<Option<OutboundMessage>, ChannelError> + Send + Sync + 'static,
{
    handler: F,
}

impl<F> FnMessageHandler<F>
where
    F: Fn(ChannelMessage) -> Result<Option<OutboundMessage>, ChannelError> + Send + Sync + 'static,
{
    pub fn new(handler: F) -> Self {
        Self { handler }
    }
}

#[async_trait]
impl<F> MessageHandler for FnMessageHandler<F>
where
    F: Fn(ChannelMessage) -> Result<Option<OutboundMessage>, ChannelError> + Send + Sync + 'static,
{
    async fn handle_message(
        &self,
        message: ChannelMessage,
    ) -> Result<Option<OutboundMessage>, ChannelError> {
        (self.handler)(message)
    }
}

/// 渠道适配器接口
#[async_trait]
pub trait ChannelAdapter: Send + Sync {
    /// 渠道名称
    fn name(&self) -> &str;

    /// 渠道类型
    fn channel_type(&self) -> ChannelType;

    /// 连接到渠道
    async fn connect(&self, config: &ChannelConfig) -> Result<(), ChannelError>;

    /// 断开连接
    async fn disconnect(&self) -> Result<(), ChannelError>;

    /// 是否已连接
    fn is_connected(&self) -> bool;

    /// 发送消息
    async fn send(&self, message: &OutboundMessage) -> Result<String, ChannelError>;

    /// 启动消息轮询
    async fn start_polling(&self, handler: Arc<dyn MessageHandler>) -> Result<(), ChannelError>;

    /// 停止轮询
    async fn stop_polling(&self) -> Result<(), ChannelError>;

    /// 获取状态
    fn status(&self) -> ChannelStatus;
}
