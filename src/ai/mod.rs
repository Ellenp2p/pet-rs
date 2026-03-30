//! AI Provider 模块
//!
//! 提供 AI 提供商管理、适配器、计费、速率限制等功能。

pub mod adapters;
pub mod budget;
pub mod crypto;
pub mod error;
pub mod manager;
pub mod pricing;
pub mod provider;
pub mod rate_limiter;
pub mod usage;

#[cfg(feature = "wasm-plugin")]
pub mod wasm_plugin;

pub use error::AIError;
pub use manager::AIProviderManager;
pub use provider::{
    AIProvider, ChatMessage, ChatResponse, ProviderConfig, ProviderType, TokenUsage,
};

#[cfg(feature = "wasm-plugin")]
pub use wasm_plugin::{WasmAIPlugin, WasmAIPluginManager};
