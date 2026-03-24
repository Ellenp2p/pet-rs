use pet_rs::prelude::*;

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

        registry.register_fn("on_spawn", move |_ctx| {
            c.fetch_add(1, Ordering::SeqCst);
        });

        registry.trigger("on_spawn", &HookContext { entity: 0 });

        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_hook_multiple_subscribers() {
        let mut registry = HookRegistry::default();
        let counter = Arc::new(AtomicU32::new(0));

        for _ in 0..3 {
            let c = counter.clone();
            registry.register_fn("on_tick", move |_ctx| {
                c.fetch_add(1, Ordering::SeqCst);
            });
        }

        registry.trigger("on_tick", &HookContext { entity: 0 });

        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn test_hook_separate_keys() {
        let mut registry = HookRegistry::default();
        let a = Arc::new(AtomicU32::new(0));
        let b = Arc::new(AtomicU32::new(0));

        let a2 = a.clone();
        let b2 = b.clone();

        registry.register_fn("key_a", move |_| {
            a2.fetch_add(1, Ordering::SeqCst);
        });
        registry.register_fn("key_b", move |_| {
            b2.fetch_add(10, Ordering::SeqCst);
        });

        let ctx = HookContext { entity: 0 };
        registry.trigger("key_a", &ctx);
        registry.trigger("key_b", &ctx);

        assert_eq!(a.load(Ordering::SeqCst), 1);
        assert_eq!(b.load(Ordering::SeqCst), 10);
    }

    #[test]
    fn test_hook_clear() {
        let mut registry = HookRegistry::default();
        registry.register_fn("test", |_ctx| {});
        assert_eq!(registry.count("test"), 1);

        registry.clear("test");
        assert_eq!(registry.count("test"), 0);
    }

    #[test]
    fn test_hook_no_trigger_wrong_key() {
        let mut registry = HookRegistry::default();
        let counter = Arc::new(AtomicU32::new(0));
        let c = counter.clone();

        registry.register_fn("real_key", move |_| {
            c.fetch_add(1, Ordering::SeqCst);
        });

        registry.trigger("wrong_key", &HookContext { entity: 0 });

        assert_eq!(counter.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn test_hook_trigger_with_no_callbacks() {
        let registry = HookRegistry::default();
        // Should not panic when triggering a key with no callbacks
        registry.trigger("nonexistent", &HookContext { entity: 0 });
    }

    #[test]
    fn test_hook_clear_all() {
        let mut registry = HookRegistry::default();
        registry.register_fn("a", |_ctx| {});
        registry.register_fn("b", |_ctx| {});
        assert!(registry.count("a") > 0);
        assert!(registry.count("b") > 0);

        registry.clear_all();
        assert_eq!(registry.count("a"), 0);
        assert_eq!(registry.count("b"), 0);
    }
}

#[cfg(test)]
mod network_channel_tests {
    use pet_rs::network::NetworkChannel;

    #[test]
    fn test_generic_channel_i32() {
        let channel: NetworkChannel<i32> = NetworkChannel::default();
        channel.send(42).unwrap();
        let msgs = channel.drain_outgoing().unwrap();
        assert_eq!(msgs, vec![42]);
    }

    #[test]
    fn test_generic_channel_string() {
        let channel: NetworkChannel<String> = NetworkChannel::default();
        channel.send("hello".into()).unwrap();
        channel.send("world".into()).unwrap();
        let msgs = channel.drain_outgoing().unwrap();
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0], "hello");
        assert_eq!(msgs[1], "world");
    }

    #[test]
    fn test_channel_incoming() {
        let channel: NetworkChannel<u64> = NetworkChannel::default();
        channel.inject_incoming(100).unwrap();
        channel.inject_incoming(200).unwrap();
        let msgs = channel.drain_incoming().unwrap();
        assert_eq!(msgs, vec![100, 200]);
    }

    #[test]
    fn test_channel_drain_clears() {
        let channel: NetworkChannel<&str> = NetworkChannel::default();
        channel.send("a").unwrap();
        let first = channel.drain_outgoing().unwrap();
        assert_eq!(first.len(), 1);

        let second = channel.drain_outgoing().unwrap();
        assert!(second.is_empty());
    }

    #[test]
    fn test_channel_with_struct() {
        #[derive(Debug, Clone, PartialEq)]
        struct Dto {
            id: u64,
            value: f32,
        }

        let channel: NetworkChannel<Dto> = NetworkChannel::default();
        channel
            .send(Dto {
                id: 1,
                value: std::f32::consts::PI,
            })
            .unwrap();
        let msgs = channel.drain_outgoing().unwrap();
        assert_eq!(msgs[0].id, 1);
    }

    #[test]
    fn test_channel_send_returns_ok() {
        let channel: NetworkChannel<i32> = NetworkChannel::default();
        let result = channel.send(1);
        assert!(result.is_ok());
    }

    #[test]
    fn test_channel_drain_returns_ok() {
        let channel: NetworkChannel<i32> = NetworkChannel::default();
        assert!(channel.drain_outgoing().is_ok());
        assert!(channel.drain_incoming().is_ok());
    }
}

#[cfg(test)]
mod error_tests {
    use pet_rs::error::FrameworkError;

    #[test]
    fn test_error_display() {
        let err = FrameworkError::LockPoisoned;
        assert_eq!(err.to_string(), "resource lock poisoned");

        let err = FrameworkError::ChannelClosed("test".into());
        assert!(err.to_string().contains("test"));

        let err = FrameworkError::Plugin("crashed".into());
        assert!(err.to_string().contains("crashed"));
    }
}

#[cfg(test)]
mod config_tests {
    use pet_rs::network::NetworkConfig;

    #[test]
    fn test_default_config() {
        let config = NetworkConfig::default();
        assert_eq!(config.server_url, "http://localhost:3000");
        assert!(!config.use_websocket);
    }
}
