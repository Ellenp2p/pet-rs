//! 插件生命周期管理器
//!
//! 负责管理插件的安装、升级、卸载等生命周期操作。

use crate::error::FrameworkError;

/// 生命周期 Hook
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LifecycleHook {
    /// 安装前
    PreInstall,
    /// 安装后
    PostInstall,
    /// 升级前
    PreUpgrade,
    /// 升级后
    PostUpgrade,
    /// 卸载前
    PreUninstall,
    /// 卸载后
    PostUninstall,
    /// 启用前
    PreEnable,
    /// 启用后
    PostEnable,
    /// 禁用前
    PreDisable,
    /// 禁用后
    PostDisable,
}

impl LifecycleHook {
    /// 获取 Hook 名称
    pub fn name(&self) -> &str {
        match self {
            LifecycleHook::PreInstall => "pre_install",
            LifecycleHook::PostInstall => "post_install",
            LifecycleHook::PreUpgrade => "pre_upgrade",
            LifecycleHook::PostUpgrade => "post_upgrade",
            LifecycleHook::PreUninstall => "pre_uninstall",
            LifecycleHook::PostUninstall => "post_uninstall",
            LifecycleHook::PreEnable => "pre_enable",
            LifecycleHook::PostEnable => "post_enable",
            LifecycleHook::PreDisable => "pre_disable",
            LifecycleHook::PostDisable => "post_disable",
        }
    }
}

/// 生命周期事件
#[derive(Debug, Clone)]
pub struct LifecycleEvent {
    /// Hook 类型
    pub hook: LifecycleHook,
    /// 插件 ID
    pub plugin_id: String,
    /// 事件数据
    pub data: serde_json::Value,
}

/// 生命周期回调
pub type LifecycleCallback =
    Box<dyn Fn(&LifecycleEvent) -> Result<(), FrameworkError> + Send + Sync>;

/// 生命周期管理器
pub struct PluginLifecycleManager {
    /// 生命周期回调
    callbacks: std::collections::HashMap<LifecycleHook, Vec<LifecycleCallback>>,
}

impl PluginLifecycleManager {
    /// 创建新的生命周期管理器
    pub fn new() -> Self {
        Self {
            callbacks: std::collections::HashMap::new(),
        }
    }

    /// 注册生命周期回调
    pub fn register_callback(&mut self, hook: LifecycleHook, callback: LifecycleCallback) {
        self.callbacks
            .entry(hook)
            .or_insert_with(Vec::new)
            .push(callback);
    }

    /// 触发生命周期事件
    pub fn trigger(&self, event: &LifecycleEvent) -> Result<(), FrameworkError> {
        if let Some(callbacks) = self.callbacks.get(&event.hook) {
            for callback in callbacks {
                callback(event)?;
            }
        }
        Ok(())
    }

    /// 安装插件
    pub fn install(&self, plugin_id: &str) -> Result<(), FrameworkError> {
        let event = LifecycleEvent {
            hook: LifecycleHook::PreInstall,
            plugin_id: plugin_id.to_string(),
            data: serde_json::json!({}),
        };
        self.trigger(&event)?;

        // 实际安装逻辑
        log::info!("Installing plugin: {}", plugin_id);

        let event = LifecycleEvent {
            hook: LifecycleHook::PostInstall,
            plugin_id: plugin_id.to_string(),
            data: serde_json::json!({}),
        };
        self.trigger(&event)?;

        Ok(())
    }

    /// 卸载插件
    pub fn uninstall(&self, plugin_id: &str) -> Result<(), FrameworkError> {
        let event = LifecycleEvent {
            hook: LifecycleHook::PreUninstall,
            plugin_id: plugin_id.to_string(),
            data: serde_json::json!({}),
        };
        self.trigger(&event)?;

        // 实际卸载逻辑
        log::info!("Uninstalling plugin: {}", plugin_id);

        let event = LifecycleEvent {
            hook: LifecycleHook::PostUninstall,
            plugin_id: plugin_id.to_string(),
            data: serde_json::json!({}),
        };
        self.trigger(&event)?;

        Ok(())
    }

    /// 启用插件
    pub fn enable(&self, plugin_id: &str) -> Result<(), FrameworkError> {
        let event = LifecycleEvent {
            hook: LifecycleHook::PreEnable,
            plugin_id: plugin_id.to_string(),
            data: serde_json::json!({}),
        };
        self.trigger(&event)?;

        // 实际启用逻辑
        log::info!("Enabling plugin: {}", plugin_id);

        let event = LifecycleEvent {
            hook: LifecycleHook::PostEnable,
            plugin_id: plugin_id.to_string(),
            data: serde_json::json!({}),
        };
        self.trigger(&event)?;

        Ok(())
    }

    /// 禁用插件
    pub fn disable(&self, plugin_id: &str) -> Result<(), FrameworkError> {
        let event = LifecycleEvent {
            hook: LifecycleHook::PreDisable,
            plugin_id: plugin_id.to_string(),
            data: serde_json::json!({}),
        };
        self.trigger(&event)?;

        // 实际禁用逻辑
        log::info!("Disabling plugin: {}", plugin_id);

        let event = LifecycleEvent {
            hook: LifecycleHook::PostDisable,
            plugin_id: plugin_id.to_string(),
            data: serde_json::json!({}),
        };
        self.trigger(&event)?;

        Ok(())
    }
}

impl Default for PluginLifecycleManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[test]
    fn test_lifecycle_manager() {
        let mut manager = PluginLifecycleManager::new();
        let counter = Arc::new(AtomicU32::new(0));
        let c = counter.clone();

        manager.register_callback(
            LifecycleHook::PostInstall,
            Box::new(move |_| {
                c.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }),
        );

        manager.install("test-plugin").unwrap();
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }
}
