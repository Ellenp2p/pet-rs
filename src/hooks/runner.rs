//! Hook 执行器
//!
//! 负责执行 Hook 并处理结果。

use super::context::{HookContext, HookResult};
use super::points::{HookExecutionMode, HookPoint};
use super::registry::HookRegistry;
use crate::error::FrameworkError;

/// Hook 执行器
pub struct HookRunner {
    /// Hook 注册表
    registry: HookRegistry,
}

impl HookRunner {
    /// 创建新的 Hook 执行器
    pub fn new(registry: HookRegistry) -> Self {
        Self { registry }
    }

    /// 获取 Hook 注册表
    pub fn registry(&self) -> &HookRegistry {
        &self.registry
    }

    /// 获取可变 Hook 注册表
    pub fn registry_mut(&mut self) -> &mut HookRegistry {
        &mut self.registry
    }

    /// 执行 Hook
    pub fn run(
        &self,
        hook_point: HookPoint,
        context: &HookContext,
    ) -> Result<Vec<HookResult>, FrameworkError> {
        let registrations = self.registry.get_enabled_registrations(hook_point);

        if registrations.is_empty() {
            return Ok(vec![HookResult::Continue]);
        }

        match hook_point.execution_mode() {
            HookExecutionMode::Sequential => self.run_sequential(registrations, context),
            HookExecutionMode::Parallel => self.run_parallel(registrations, context),
            HookExecutionMode::Exclusive => self.run_exclusive(registrations, context),
        }
    }

    /// 顺序执行 Hook
    fn run_sequential(
        &self,
        registrations: Vec<&super::registry::HookRegistration>,
        context: &HookContext,
    ) -> Result<Vec<HookResult>, FrameworkError> {
        let mut results = Vec::new();
        let mut current_context = context.clone();

        for registration in registrations {
            let result = (registration.callback)(&current_context)?;

            // 如果被阻止，停止执行
            if result.is_blocked() {
                results.push(result);
                break;
            }

            // 如果有修改，更新上下文
            if let Some(modified_data) = result.modified_data() {
                current_context.set_data("modified".to_string(), modified_data.clone());
            }

            results.push(result);
        }

        Ok(results)
    }

    /// 并行执行 Hook
    fn run_parallel(
        &self,
        registrations: Vec<&super::registry::HookRegistration>,
        context: &HookContext,
    ) -> Result<Vec<HookResult>, FrameworkError> {
        let mut results = Vec::new();

        for registration in registrations {
            let result = (registration.callback)(context)?;
            results.push(result);
        }

        Ok(results)
    }

    /// 独占执行 Hook
    fn run_exclusive(
        &self,
        registrations: Vec<&super::registry::HookRegistration>,
        context: &HookContext,
    ) -> Result<Vec<HookResult>, FrameworkError> {
        if registrations.is_empty() {
            return Ok(vec![HookResult::Continue]);
        }

        // 只执行第一个（优先级最高的）
        let registration = &registrations[0];
        let result = (registration.callback)(context)?;
        Ok(vec![result])
    }

    /// 触发 Hook（简化版本）
    pub fn trigger(
        &self,
        hook_point: HookPoint,
        context: &HookContext,
    ) -> Result<HookResult, FrameworkError> {
        let results = self.run(hook_point, context)?;

        // 合并结果
        if results.is_empty() {
            return Ok(HookResult::Continue);
        }

        // 检查是否有被阻止的
        for result in &results {
            if result.is_blocked() {
                return Ok(result.clone());
            }
        }

        // 检查是否有修改的
        for result in &results {
            if let HookResult::Modified(data) = result {
                return Ok(HookResult::Modified(data.clone()));
            }
        }

        // 检查是否有跳过的
        for result in &results {
            if result.should_skip() {
                return Ok(HookResult::Skip);
            }
        }

        // 检查是否有替换的
        for result in &results {
            if let HookResult::Replace(data) = result {
                return Ok(HookResult::Replace(data.clone()));
            }
        }

        Ok(HookResult::Continue)
    }

    /// 触发 Hook（使用默认上下文）
    pub fn trigger_simple(
        &self,
        hook_point: HookPoint,
        agent_id: &str,
    ) -> Result<HookResult, FrameworkError> {
        let context = HookContext::new(hook_point, agent_id.to_string());
        self.trigger(hook_point, &context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_hook_runner_creation() {
        let registry = HookRegistry::new();
        let runner = HookRunner::new(registry);
        assert_eq!(runner.registry().total_count(), 0);
    }

    #[test]
    fn test_hook_runner_sequential() {
        let mut registry = HookRegistry::new();

        let callback: super::super::registry::HookCallback = Arc::new(|ctx| {
            assert_eq!(ctx.agent_id, "test-agent");
            Ok(HookResult::Continue)
        });

        registry
            .register(HookPoint::OnInputReceived, 100, callback)
            .unwrap();

        let runner = HookRunner::new(registry);
        let context = HookContext::new(HookPoint::OnInputReceived, "test-agent".to_string());

        let results = runner.run(HookPoint::OnInputReceived, &context).unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].should_continue());
    }

    #[test]
    fn test_hook_runner_blocked() {
        let mut registry = HookRegistry::new();

        let callback: super::super::registry::HookCallback = Arc::new(|_| {
            Ok(HookResult::Blocked {
                reason: "test block".to_string(),
            })
        });

        registry
            .register(HookPoint::BeforeDecision, 100, callback)
            .unwrap();

        let runner = HookRunner::new(registry);
        let context = HookContext::new(HookPoint::BeforeDecision, "test-agent".to_string());

        let result = runner.trigger(HookPoint::BeforeDecision, &context).unwrap();
        assert!(result.is_blocked());
    }

    #[test]
    fn test_hook_runner_modified() {
        let mut registry = HookRegistry::new();

        let callback: super::super::registry::HookCallback =
            Arc::new(|_| Ok(HookResult::Modified(serde_json::json!({"modified": true}))));

        registry
            .register(HookPoint::BeforeAction, 100, callback)
            .unwrap();

        let runner = HookRunner::new(registry);
        let context = HookContext::new(HookPoint::BeforeAction, "test-agent".to_string());

        let result = runner.trigger(HookPoint::BeforeAction, &context).unwrap();
        assert!(result.modified_data().is_some());
    }
}
