//! 渠道插件管理器

#[cfg(feature = "wasm-plugin")]
use std::collections::HashMap;
#[cfg(feature = "wasm-plugin")]
use std::path::Path;
#[cfg(feature = "wasm-plugin")]
use std::sync::Arc;
#[cfg(feature = "wasm-plugin")]
use tokio::sync::RwLock;

#[cfg(feature = "wasm-plugin")]
use crate::wasm::WasmtimePlugin;

#[cfg(feature = "wasm-plugin")]
use super::adapter::{ChannelAdapter, ChannelConfig, ChannelError, ChannelStatus, MessageHandler};
#[cfg(feature = "wasm-plugin")]
use super::wasm_channel::WasmChannelPlugin;

/// 渠道插件管理器
#[cfg(feature = "wasm-plugin")]
pub struct ChannelPluginManager {
    /// 已注册的渠道插件
    plugins: Arc<RwLock<HashMap<String, Arc<WasmChannelPlugin>>>>,
    /// 消息处理器
    handler: Arc<dyn MessageHandler>,
}

#[cfg(feature = "wasm-plugin")]
impl ChannelPluginManager {
    /// 创建新的管理器
    pub fn new(handler: Arc<dyn MessageHandler>) -> Self {
        Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
            handler,
        }
    }

    /// 注册 WASM 渠道插件
    pub async fn register(
        &self,
        name: &str,
        wasm_path: &Path,
        config: ChannelConfig,
    ) -> Result<(), ChannelError> {
        // 检查是否已注册
        {
            let plugins = self.plugins.read().await;
            if plugins.contains_key(name) {
                return Err(ChannelError::Unknown(format!(
                    "Channel '{}' already registered",
                    name
                )));
            }
        }

        // 加载 WASM 插件
        let plugin = WasmtimePlugin::load(wasm_path, Some(name.to_string()))
            .map_err(|e| ChannelError::WasmError(e.to_string()))?;

        // 创建渠道插件
        let channel = Arc::new(WasmChannelPlugin::new(name.to_string(), plugin));

        // 注册插件
        {
            let mut plugins = self.plugins.write().await;
            plugins.insert(name.to_string(), channel);
        }

        log::info!("Channel '{}' registered from {:?}", name, wasm_path);
        Ok(())
    }

    /// 注销渠道插件
    pub async fn unregister(&self, name: &str) -> Result<(), ChannelError> {
        let mut plugins = self.plugins.write().await;
        if let Some(channel) = plugins.remove(name) {
            // 断开连接
            let _ = channel.disconnect().await;
            log::info!("Channel '{}' unregistered", name);
            Ok(())
        } else {
            Err(ChannelError::Unknown(format!(
                "Channel '{}' not found",
                name
            )))
        }
    }

    /// 连接指定渠道
    pub async fn connect(&self, name: &str, config: &ChannelConfig) -> Result<(), ChannelError> {
        let plugins = self.plugins.read().await;
        if let Some(channel) = plugins.get(name) {
            channel.connect(config).await?;
            channel.start_polling(self.handler.clone()).await?;
            log::info!("Channel '{}' connected", name);
            Ok(())
        } else {
            Err(ChannelError::Unknown(format!(
                "Channel '{}' not found",
                name
            )))
        }
    }

    /// 断开指定渠道
    pub async fn disconnect(&self, name: &str) -> Result<(), ChannelError> {
        let plugins = self.plugins.read().await;
        if let Some(channel) = plugins.get(name) {
            channel.disconnect().await?;
            log::info!("Channel '{}' disconnected", name);
            Ok(())
        } else {
            Err(ChannelError::Unknown(format!(
                "Channel '{}' not found",
                name
            )))
        }
    }

    /// 连接所有渠道
    pub async fn connect_all(
        &self,
        configs: &HashMap<String, ChannelConfig>,
    ) -> Result<(), ChannelError> {
        let plugins = self.plugins.read().await;
        for (name, channel) in plugins.iter() {
            if let Some(config) = configs.get(name) {
                if config.enabled {
                    match channel.connect(config).await {
                        Ok(()) => {
                            let _ = channel.start_polling(self.handler.clone()).await;
                            log::info!("Channel '{}' connected", name);
                        }
                        Err(e) => {
                            log::error!("Failed to connect channel '{}': {}", name, e);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// 断开所有渠道
    pub async fn disconnect_all(&self) -> Result<(), ChannelError> {
        let plugins = self.plugins.read().await;
        for (name, channel) in plugins.iter() {
            if channel.is_connected() {
                let _ = channel.disconnect().await;
                log::info!("Channel '{}' disconnected", name);
            }
        }
        Ok(())
    }

    /// 发送消息到指定渠道
    pub async fn send(
        &self,
        channel_name: &str,
        message: &super::message::OutboundMessage,
    ) -> Result<String, ChannelError> {
        let plugins = self.plugins.read().await;
        if let Some(channel) = plugins.get(channel_name) {
            channel.send(message).await
        } else {
            Err(ChannelError::Unknown(format!(
                "Channel '{}' not found",
                channel_name
            )))
        }
    }

    /// 获取指定渠道状态
    pub async fn status(&self, name: &str) -> Option<ChannelStatus> {
        let plugins = self.plugins.read().await;
        plugins.get(name).map(|c| c.status())
    }

    /// 获取所有渠道状态
    pub async fn status_all(&self) -> HashMap<String, ChannelStatus> {
        let plugins = self.plugins.read().await;
        plugins
            .iter()
            .map(|(name, channel)| (name.clone(), channel.status()))
            .collect()
    }

    /// 列出所有已注册渠道
    pub async fn list(&self) -> Vec<String> {
        let plugins = self.plugins.read().await;
        plugins.keys().cloned().collect()
    }
}
