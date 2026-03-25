//! Hook 注册表
//!
//! 管理 Hook 的注册和查找。

use std::collections::HashMap;
use std::sync::Arc;

use super::context::{HookContext, HookResult};
use super::points::HookPoint;

/// Hook 回调函数类型
pub type HookCallback =
    Arc<dyn Fn(&HookContext) -> Result<HookResult, crate::error::FrameworkError> + Send + Sync>;

/// Hook 注册信息
#[derive(Clone)]
pub struct HookRegistration {
    /// Hook 点
    pub hook_point: HookPoint,
    /// 优先级（数字越小优先级越高）
    pub priority: i32,
    /// 回调函数
    pub callback: HookCallback,
    /// 插件 ID（如果有）
    pub plugin_id: Option<String>,
    /// 是否启用
    pub enabled: bool,
}

/// Hook 注册表
#[derive(Default)]
pub struct HookRegistry {
    /// Hook 注册信息（按 Hook 点分组）
    hooks: HashMap<HookPoint, Vec<HookRegistration>>,
}

impl HookRegistry {
    /// 创建新的 Hook 注册表
    pub fn new() -> Self {
        Self {
            hooks: HashMap::new(),
        }
    }

    /// 注册 Hook
    pub fn register(
        &mut self,
        hook_point: HookPoint,
        priority: i32,
        callback: HookCallback,
    ) -> Result<(), crate::error::FrameworkError> {
        let registration = HookRegistration {
            hook_point,
            priority,
            callback,
            plugin_id: None,
            enabled: true,
        };

        self.hooks.entry(hook_point).or_default().push(registration);

        // 按优先级排序
        if let Some(hooks) = self.hooks.get_mut(&hook_point) {
            hooks.sort_by_key(|h| h.priority);
        }

        Ok(())
    }

    /// 注册 Hook（带插件 ID）
    pub fn register_with_plugin(
        &mut self,
        hook_point: HookPoint,
        priority: i32,
        callback: HookCallback,
        plugin_id: String,
    ) -> Result<(), crate::error::FrameworkError> {
        let registration = HookRegistration {
            hook_point,
            priority,
            callback,
            plugin_id: Some(plugin_id),
            enabled: true,
        };

        self.hooks.entry(hook_point).or_default().push(registration);

        // 按优先级排序
        if let Some(hooks) = self.hooks.get_mut(&hook_point) {
            hooks.sort_by_key(|h| h.priority);
        }

        Ok(())
    }

    /// 注销 Hook
    pub fn unregister(
        &mut self,
        hook_point: HookPoint,
        plugin_id: &str,
    ) -> Result<(), crate::error::FrameworkError> {
        if let Some(hooks) = self.hooks.get_mut(&hook_point) {
            hooks.retain(|h| h.plugin_id.as_deref() != Some(plugin_id));
        }
        Ok(())
    }

    /// 获取 Hook 点的所有注册信息
    pub fn get_registrations(&self, hook_point: HookPoint) -> &[HookRegistration] {
        self.hooks
            .get(&hook_point)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// 获取启用的 Hook 注册信息
    pub fn get_enabled_registrations(&self, hook_point: HookPoint) -> Vec<&HookRegistration> {
        self.get_registrations(hook_point)
            .iter()
            .filter(|r| r.enabled)
            .collect()
    }

    /// 启用 Hook
    pub fn enable(
        &mut self,
        hook_point: HookPoint,
        plugin_id: &str,
    ) -> Result<(), crate::error::FrameworkError> {
        if let Some(hooks) = self.hooks.get_mut(&hook_point) {
            for hook in hooks.iter_mut() {
                if hook.plugin_id.as_deref() == Some(plugin_id) {
                    hook.enabled = true;
                }
            }
        }
        Ok(())
    }

    /// 禁用 Hook
    pub fn disable(
        &mut self,
        hook_point: HookPoint,
        plugin_id: &str,
    ) -> Result<(), crate::error::FrameworkError> {
        if let Some(hooks) = self.hooks.get_mut(&hook_point) {
            for hook in hooks.iter_mut() {
                if hook.plugin_id.as_deref() == Some(plugin_id) {
                    hook.enabled = false;
                }
            }
        }
        Ok(())
    }

    /// 获取 Hook 点的数量
    pub fn count(&self, hook_point: HookPoint) -> usize {
        self.hooks.get(&hook_point).map(|v| v.len()).unwrap_or(0)
    }

    /// 获取所有 Hook 点的数量
    pub fn total_count(&self) -> usize {
        self.hooks.values().map(|v| v.len()).sum()
    }

    /// 清空所有 Hook
    pub fn clear(&mut self) {
        self.hooks.clear();
    }

    /// 清空指定 Hook 点的所有 Hook
    pub fn clear_hook_point(&mut self, hook_point: HookPoint) {
        self.hooks.remove(&hook_point);
    }

    /// 触发 Hook（便捷方法）
    pub fn trigger(
        &self,
        hook_name: &str,
        context: &HookContext,
    ) -> Result<HookResult, crate::error::FrameworkError> {
        // 解析 Hook 点
        let hook_point = HookPoint::from_name(hook_name).ok_or_else(|| {
            crate::error::FrameworkError::Other(format!("Unknown hook: {}", hook_name))
        })?;

        // 获取启用的 Hook 注册信息
        let registrations = self.get_enabled_registrations(hook_point);

        if registrations.is_empty() {
            return Ok(HookResult::Continue);
        }

        // 执行第一个（优先级最高的）Hook
        let registration = &registrations[0];
        (registration.callback)(context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_registry_creation() {
        let registry = HookRegistry::new();
        assert_eq!(registry.total_count(), 0);
    }

    #[test]
    fn test_hook_registry_register() {
        let mut registry = HookRegistry::new();
        let callback: HookCallback = Arc::new(|_| Ok(HookResult::Continue));

        registry
            .register(HookPoint::OnInputReceived, 100, callback)
            .unwrap();

        assert_eq!(registry.count(HookPoint::OnInputReceived), 1);
    }

    #[test]
    fn test_hook_registry_priority() {
        let mut registry = HookRegistry::new();

        let callback1: HookCallback = Arc::new(|_| Ok(HookResult::Continue));
        let callback2: HookCallback = Arc::new(|_| Ok(HookResult::Continue));

        registry
            .register(HookPoint::OnInputReceived, 200, callback1)
            .unwrap();
        registry
            .register(HookPoint::OnInputReceived, 100, callback2)
            .unwrap();

        let registrations = registry.get_registrations(HookPoint::OnInputReceived);
        assert_eq!(registrations.len(), 2);
        assert_eq!(registrations[0].priority, 100);
        assert_eq!(registrations[1].priority, 200);
    }

    #[test]
    fn test_hook_registry_enable_disable() {
        let mut registry = HookRegistry::new();
        let callback: HookCallback = Arc::new(|_| Ok(HookResult::Continue));

        registry
            .register_with_plugin(
                HookPoint::OnInputReceived,
                100,
                callback,
                "plugin-1".to_string(),
            )
            .unwrap();

        registry
            .disable(HookPoint::OnInputReceived, "plugin-1")
            .unwrap();

        let enabled = registry.get_enabled_registrations(HookPoint::OnInputReceived);
        assert_eq!(enabled.len(), 0);

        registry
            .enable(HookPoint::OnInputReceived, "plugin-1")
            .unwrap();

        let enabled = registry.get_enabled_registrations(HookPoint::OnInputReceived);
        assert_eq!(enabled.len(), 1);
    }
}
