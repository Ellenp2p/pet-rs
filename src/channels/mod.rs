//! 渠道模块
//!
//! 提供统一的渠道适配器接口和 WASM 渠道插件支持。

pub mod adapter;
pub mod manager;
pub mod message;
#[cfg(feature = "wasm-plugin")]
pub mod wasm_channel;

pub use adapter::{
    ChannelAdapter, ChannelConfig, ChannelError, ChannelStatus, ChannelType, MessageHandler,
};
#[cfg(feature = "wasm-plugin")]
pub use manager::ChannelPluginManager;
pub use message::{
    ChannelMessage, MediaType, MessageContent, OutboundMessage, SenderInfo, SessionId,
};
#[cfg(feature = "wasm-plugin")]
pub use wasm_channel::WasmChannelPlugin;
