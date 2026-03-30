//! AI 错误类型

use thiserror::Error;

/// AI 相关错误
#[derive(Debug, Error)]
pub enum AIError {
    #[error("网络错误: {0}")]
    NetworkError(String),

    #[error("API 错误: {0}")]
    ApiError(String),

    #[error("解析错误: {0}")]
    ParseError(String),

    #[error("空响应")]
    EmptyResponse,

    #[error("未知提供商: {0}")]
    UnknownProvider(String),

    #[error("没有可用的提供商")]
    NoProviderAvailable,

    #[error("所有提供商都失败")]
    AllProvidersFailed,

    #[error("配置错误: {0}")]
    ConfigError(String),

    #[error("解密错误: {0}")]
    DecryptionError(String),

    #[error("速率限制: {0}")]
    RateLimited(String),

    #[error("预算超限: {0}")]
    BudgetExceeded(String),

    #[error("连接测试失败: {0}")]
    ConnectionTestFailed(String),

    #[error("WASM 插件错误: {0}")]
    WasmPluginError(String),
}
