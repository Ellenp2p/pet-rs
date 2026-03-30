//! WASM AI Plugin 包装器
//!
//! 将 WASM 插件包装为 AIProvider trait 实现。

#[cfg(feature = "wasm-plugin")]
use crate::ai::error::AIError;
#[cfg(feature = "wasm-plugin")]
use crate::ai::provider::{AIProvider, ChatMessage, ChatResponse, ProviderConfig, ProviderType};
#[cfg(feature = "wasm-plugin")]
use crate::wasm::{WasmPlugin, WasmPluginId};

/// WASM AI 插件包装器
#[cfg(feature = "wasm-plugin")]
pub struct WasmAIPlugin {
    /// 插件名称
    name: String,
    /// 支持的模型
    models: Vec<String>,
    /// WASM 插件实例
    plugin: Box<dyn WasmPlugin>,
}

#[cfg(feature = "wasm-plugin")]
impl WasmAIPlugin {
    /// 创建新的 WASM AI 插件
    pub fn new(name: String, models: Vec<String>, plugin: Box<dyn WasmPlugin>) -> Self {
        Self {
            name,
            models,
            plugin,
        }
    }

    /// 获取插件 ID
    pub fn plugin_id(&self) -> &WasmPluginId {
        self.plugin.id()
    }
}

#[cfg(feature = "wasm-plugin")]
impl AIProvider for WasmAIPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn provider_type(&self) -> ProviderType {
        ProviderType::Custom
    }

    fn supported_models(&self) -> Vec<String> {
        self.models.clone()
    }

    fn chat(
        &self,
        _messages: Vec<ChatMessage>,
        _config: &ProviderConfig,
    ) -> Result<ChatResponse, AIError> {
        // 构建请求 JSON
        // 通过事件发送请求到 WASM 插件
        // WASM 插件会使用 host_http_request 进行实际的 HTTP 请求
        // 响应通过 wasm_plugin_set_data 返回

        // 这里简化实现，实际应该调用 WASM 插件的特定函数
        // 并等待响应

        Err(AIError::WasmPluginError(
            "WASM AI 插件聊天功能需要在 WASM 插件中实现 wasm_ai_chat 导出函数".to_string(),
        ))
    }

    fn chat_stream(
        &self,
        _messages: Vec<ChatMessage>,
        _config: &ProviderConfig,
        _on_chunk: Box<dyn Fn(String) + Send>,
    ) -> Result<ChatResponse, AIError> {
        // 流式聊天实现
        // 需要 WASM 插件支持 wasm_ai_chat_stream 导出函数
        // 并通过 host_emit_chunk 回调返回片段

        Err(AIError::WasmPluginError(
            "WASM AI 插件流式聊天功能需要在 WASM 插件中实现 wasm_ai_chat_stream 导出函数"
                .to_string(),
        ))
    }
}

/// WASM AI 插件管理器
#[cfg(feature = "wasm-plugin")]
pub struct WasmAIPluginManager {
    plugins: std::collections::HashMap<String, WasmAIPlugin>,
}

#[cfg(feature = "wasm-plugin")]
impl WasmAIPluginManager {
    pub fn new() -> Self {
        Self {
            plugins: std::collections::HashMap::new(),
        }
    }

    /// 注册 WASM AI 插件
    pub fn register(&mut self, plugin: WasmAIPlugin) -> Result<(), AIError> {
        let name = plugin.name().to_string();
        self.plugins.insert(name, plugin);
        Ok(())
    }

    /// 注销 WASM AI 插件
    pub fn unregister(&mut self, name: &str) -> Option<WasmAIPlugin> {
        self.plugins.remove(name)
    }

    /// 获取插件
    pub fn get(&self, name: &str) -> Option<&WasmAIPlugin> {
        self.plugins.get(name)
    }

    /// 列出所有插件
    pub fn list(&self) -> Vec<&str> {
        self.plugins.keys().map(|s| s.as_str()).collect()
    }
}

#[cfg(feature = "wasm-plugin")]
impl Default for WasmAIPluginManager {
    fn default() -> Self {
        Self::new()
    }
}
