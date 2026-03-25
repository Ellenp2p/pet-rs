use agent_pet_rs::hooks::context::HookResult;
use agent_pet_rs::hooks::HookPoint;
use agent_pet_rs::prelude::*;

#[cfg(test)]
mod hook_tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[test]
    fn test_hook_register_and_trigger() {
        let mut registry = HookRegistry::default();
        let counter = Arc::new(AtomicU32::new(0));
        let c = counter.clone();

        registry
            .register(
                HookPoint::OnInputReceived,
                100,
                Arc::new(move |_ctx| {
                    c.fetch_add(1, Ordering::SeqCst);
                    Ok(HookResult::Continue)
                }),
            )
            .unwrap();

        let ctx = HookContext::new(HookPoint::OnInputReceived, "test-agent".to_string());
        registry.trigger("on_input_received", &ctx).unwrap();

        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_hook_multiple_subscribers() {
        let mut registry = HookRegistry::default();
        let counter = Arc::new(AtomicU32::new(0));

        for _ in 0..3 {
            let c = counter.clone();
            registry
                .register(
                    HookPoint::OnInputReceived,
                    100,
                    Arc::new(move |_ctx| {
                        c.fetch_add(1, Ordering::SeqCst);
                        Ok(HookResult::Continue)
                    }),
                )
                .unwrap();
        }

        let ctx = HookContext::new(HookPoint::OnInputReceived, "test-agent".to_string());
        registry.trigger("on_input_received", &ctx).unwrap();

        // Note: The new HookRegistry only triggers the first (highest priority) hook
        // So we expect 1, not 3
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_hook_separate_keys() {
        let mut registry = HookRegistry::default();
        let a = Arc::new(AtomicU32::new(0));
        let b = Arc::new(AtomicU32::new(0));

        let a2 = a.clone();
        let b2 = b.clone();

        registry
            .register(
                HookPoint::OnInputReceived,
                100,
                Arc::new(move |_| {
                    a2.fetch_add(1, Ordering::SeqCst);
                    Ok(HookResult::Continue)
                }),
            )
            .unwrap();
        registry
            .register(
                HookPoint::BeforeAction,
                100,
                Arc::new(move |_| {
                    b2.fetch_add(10, Ordering::SeqCst);
                    Ok(HookResult::Continue)
                }),
            )
            .unwrap();

        let ctx = HookContext::new(HookPoint::OnInputReceived, "test-agent".to_string());
        registry.trigger("on_input_received", &ctx).unwrap();

        let ctx = HookContext::new(HookPoint::BeforeAction, "test-agent".to_string());
        registry.trigger("before_action", &ctx).unwrap();

        assert_eq!(a.load(Ordering::SeqCst), 1);
        assert_eq!(b.load(Ordering::SeqCst), 10);
    }

    #[test]
    fn test_hook_count() {
        let mut registry = HookRegistry::default();
        registry
            .register(
                HookPoint::OnInputReceived,
                100,
                Arc::new(|_ctx| Ok(HookResult::Continue)),
            )
            .unwrap();
        assert_eq!(registry.count(HookPoint::OnInputReceived), 1);
    }

    #[test]
    fn test_hook_no_trigger_wrong_key() {
        let mut registry = HookRegistry::default();
        let counter = Arc::new(AtomicU32::new(0));
        let c = counter.clone();

        registry
            .register(
                HookPoint::OnInputReceived,
                100,
                Arc::new(move |_ctx| {
                    c.fetch_add(1, Ordering::SeqCst);
                    Ok(HookResult::Continue)
                }),
            )
            .unwrap();

        // Triggering a different hook should not increment the counter
        let ctx = HookContext::new(HookPoint::BeforeAction, "test-agent".to_string());
        registry.trigger("before_action", &ctx).unwrap();

        assert_eq!(counter.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn test_hook_trigger_with_no_callbacks() {
        let registry = HookRegistry::default();
        let ctx = HookContext::new(HookPoint::OnInputReceived, "test-agent".to_string());
        // Should not panic when triggering a hook with no callbacks
        let result = registry.trigger("on_input_received", &ctx).unwrap();
        assert!(matches!(result, HookResult::Continue));
    }

    #[test]
    fn test_hook_clear() {
        let mut registry = HookRegistry::default();
        registry
            .register(
                HookPoint::OnInputReceived,
                100,
                Arc::new(|_ctx| Ok(HookResult::Continue)),
            )
            .unwrap();
        assert_eq!(registry.count(HookPoint::OnInputReceived), 1);

        registry.clear_hook_point(HookPoint::OnInputReceived);
        assert_eq!(registry.count(HookPoint::OnInputReceived), 0);
    }

    #[test]
    fn test_hook_clear_all() {
        let mut registry = HookRegistry::default();
        registry
            .register(
                HookPoint::OnInputReceived,
                100,
                Arc::new(|_ctx| Ok(HookResult::Continue)),
            )
            .unwrap();
        registry
            .register(
                HookPoint::BeforeAction,
                100,
                Arc::new(|_ctx| Ok(HookResult::Continue)),
            )
            .unwrap();
        assert!(registry.total_count() > 0);

        registry.clear();
        assert_eq!(registry.total_count(), 0);
    }
}
