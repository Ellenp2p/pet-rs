use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;

pub type HookKey = Cow<'static, str>;

#[derive(Clone)]
pub struct HookContext {
    pub entity: u64,
}

pub type HookCallback = Arc<dyn Fn(&HookContext) + Send + Sync>;

/// Registry for dynamic, string-keyed event hooks.
///
/// Register hooks during plugin initialization.
/// Trigger hooks during gameplay.
///
/// Callbacks are wrapped in `Arc` so they can be cheaply cloned and
/// executed after the internal lock/borrow is released, preventing
/// deadlock if a callback recursively triggers the same hook.
#[derive(Default)]
pub struct HookRegistry {
    hooks: HashMap<HookKey, Vec<HookCallback>>,
}

impl HookRegistry {
    /// Register a callback for the given hook key.
    ///
    /// Call during plugin setup (requires `ResMut<HookRegistry>`).
    pub fn register(&mut self, key: impl Into<HookKey>, callback: HookCallback) {
        self.hooks.entry(key.into()).or_default().push(callback);
    }

    /// Convenience: register a plain function as a hook callback.
    pub fn register_fn<F>(&mut self, key: impl Into<HookKey>, f: F)
    where
        F: Fn(&HookContext) + Send + Sync + 'static,
    {
        self.register(key, Arc::new(f));
    }

    /// Trigger all callbacks registered under the given key.
    ///
    /// Callbacks are collected and the internal map is released before
    /// execution, so this method is safe even if a callback triggers
    /// another hook (re-entrancy is fine because `&self` only borrows
    /// the map immutably, and we clone the Vec out first).
    pub fn trigger(&self, key: &str, ctx: &HookContext) {
        let callbacks: Vec<HookCallback> = self
            .hooks
            .get(key)
            .map(|v| v.iter().map(Arc::clone).collect())
            .unwrap_or_default();

        for cb in &callbacks {
            cb(ctx);
        }
    }

    /// Remove all callbacks for a specific key.
    pub fn clear(&mut self, key: &str) {
        self.hooks.remove(key);
    }

    /// Remove all callbacks for all keys.
    pub fn clear_all(&mut self) {
        self.hooks.clear();
    }

    /// Return the number of callbacks registered under the given key.
    pub fn count(&self, key: &str) -> usize {
        self.hooks.get(key).map_or(0, |v| v.len())
    }
}
