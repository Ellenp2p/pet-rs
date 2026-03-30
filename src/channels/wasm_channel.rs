//! WASM 渠道插件包装器

#[cfg(feature = "wasm-plugin")]
use async_trait::async_trait;
#[cfg(feature = "wasm-plugin")]
use std::sync::Arc;
#[cfg(feature = "wasm-plugin")]
use tokio::sync::{Mutex, RwLock};
#[cfg(feature = "wasm-plugin")]
use tokio::task::JoinHandle;

#[cfg(feature = "wasm-plugin")]
use crate::wasm::{WasmPlugin, WasmtimePlugin};

#[cfg(feature = "wasm-plugin")]
use super::adapter::{
    ChannelAdapter, ChannelConfig, ChannelError, ChannelStatus, ChannelType, MessageHandler,
};
#[cfg(feature = "wasm-plugin")]
use super::message::OutboundMessage;

/// 连接状态
#[cfg(feature = "wasm-plugin")]
#[derive(Clone, Debug)]
struct ConnectionState {
    connected: bool,
    config: Option<ChannelConfig>,
    last_error: Option<String>,
}

#[cfg(feature = "wasm-plugin")]
impl Default for ConnectionState {
    fn default() -> Self {
        Self {
            connected: false,
            config: None,
            last_error: None,
        }
    }
}

/// WASM 渠道插件
#[cfg(feature = "wasm-plugin")]
pub struct WasmChannelPlugin {
    /// 插件名称
    name: String,
    /// WASM 插件实例
    plugin: Arc<WasmtimePlugin>,
    /// 连接状态
    state: Arc<RwLock<ConnectionState>>,
    /// 轮询任务句柄
    poll_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
}

#[cfg(feature = "wasm-plugin")]
impl WasmChannelPlugin {
    /// 创建新的 WASM 渠道插件
    pub fn new(name: String, plugin: WasmtimePlugin) -> Self {
        Self {
            name,
            plugin: Arc::new(plugin),
            state: Arc::new(RwLock::new(ConnectionState::default())),
            poll_handle: Arc::new(Mutex::new(None)),
        }
    }

    /// 调用 WASM 函数
    fn call_wasm_function(&self, func_name: &str, params: &str) -> Result<String, ChannelError> {
        // 通过事件系统调用 WASM 插件
        self.plugin
            .on_event(crate::wasm::WasmEntityId(0), func_name, params);

        // 从插件状态读取结果
        // 这是一个简化实现，实际应该有更复杂的回调机制
        Ok(String::new())
    }
}

#[cfg(feature = "wasm-plugin")]
#[async_trait]
impl ChannelAdapter for WasmChannelPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn channel_type(&self) -> ChannelType {
        ChannelType::Custom(self.name.clone())
    }

    async fn connect(&self, config: &ChannelConfig) -> Result<(), ChannelError> {
        // 检查是否已连接
        {
            let state = self.state.read().await;
            if state.connected {
                return Err(ChannelError::AlreadyConnected);
            }
        }

        // 调用 WASM connect 函数
        let params = serde_json::to_string(config)?;
        self.call_wasm_function("wasm_channel_connect", &params)?;

        // 更新状态
        {
            let mut state = self.state.write().await;
            state.connected = true;
            state.config = Some(config.clone());
        }

        Ok(())
    }

    async fn disconnect(&self) -> Result<(), ChannelError> {
        // 检查是否已连接
        {
            let state = self.state.read().await;
            if !state.connected {
                return Err(ChannelError::NotConnected);
            }
        }

        // 停止轮询
        self.stop_polling().await?;

        // 调用 WASM disconnect 函数
        self.call_wasm_function("wasm_channel_disconnect", "{}")?;

        // 更新状态
        {
            let mut state = self.state.write().await;
            state.connected = false;
        }

        Ok(())
    }

    fn is_connected(&self) -> bool {
        // 使用 try_read 避免阻塞
        self.state.try_read().map(|s| s.connected).unwrap_or(false)
    }

    async fn send(&self, message: &OutboundMessage) -> Result<String, ChannelError> {
        // 检查是否已连接
        {
            let state = self.state.read().await;
            if !state.connected {
                return Err(ChannelError::NotConnected);
            }
        }

        // 调用 WASM send 函数
        let params = serde_json::to_string(message)?;
        let result = self.call_wasm_function("wasm_channel_send", &params)?;

        // 解析结果获取 message_id
        let response: serde_json::Value =
            serde_json::from_str(&result).map_err(|e| ChannelError::SerializeError(e))?;

        Ok(response["message_id"].as_str().unwrap_or("").to_string())
    }

    async fn start_polling(&self, handler: Arc<dyn MessageHandler>) -> Result<(), ChannelError> {
        // 检查是否已连接
        {
            let state = self.state.read().await;
            if !state.connected {
                return Err(ChannelError::NotConnected);
            }
        }

        // 获取轮询间隔
        let interval_ms = {
            let state = self.state.read().await;
            state
                .config
                .as_ref()
                .map(|c| c.poll_interval_ms)
                .unwrap_or(1000)
        };

        // 停止现有的轮询任务
        self.stop_polling().await?;

        // 启动新的轮询任务
        let plugin = self.plugin.clone();
        let state = self.state.clone();
        let name = self.name.clone();

        let handle = tokio::spawn(async move {
            loop {
                // 检查连接状态
                {
                    let s = state.read().await;
                    if !s.connected {
                        break;
                    }
                }

                // 调用 WASM poll 函数
                plugin.on_event(crate::wasm::WasmEntityId(0), "wasm_channel_poll", "{}");

                // 等待下一次轮询
                tokio::time::sleep(tokio::time::Duration::from_millis(interval_ms)).await;
            }

            log::info!("Channel '{}' polling stopped", name);
        });

        // 保存任务句柄
        {
            let mut poll_handle = self.poll_handle.lock().await;
            *poll_handle = Some(handle);
        }

        Ok(())
    }

    async fn stop_polling(&self) -> Result<(), ChannelError> {
        let mut poll_handle = self.poll_handle.lock().await;
        if let Some(handle) = poll_handle.take() {
            handle.abort();
        }
        Ok(())
    }

    fn status(&self) -> ChannelStatus {
        let connected = self.state.try_read().map(|s| s.connected).unwrap_or(false);

        ChannelStatus {
            connected,
            pending_messages: 0,
            last_activity: 0,
            error: None,
        }
    }
}
