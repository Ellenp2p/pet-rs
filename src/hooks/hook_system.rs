use bevy::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HookType {
    OnPetSpawn,
    OnFeed,
    OnStateChange,
    OnTick,
    OnNetworkSync,
}

#[derive(Clone)]
pub struct HookContext {
    pub entity: Entity,
    pub pet_id: Option<u64>,
}

pub type HookCallback = Arc<dyn Fn(&HookContext) + Send + Sync>;

#[derive(Resource, Clone)]
pub struct HookRegistry {
    hooks: Arc<Mutex<HashMap<HookType, Vec<HookCallback>>>>,
}

impl Default for HookRegistry {
    fn default() -> Self {
        Self {
            hooks: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl HookRegistry {
    pub fn register(&self, hook_type: HookType, callback: HookCallback) {
        let mut hooks = self.hooks.lock().unwrap();
        hooks.entry(hook_type).or_default().push(callback);
    }

    pub fn register_fn<F>(&self, hook_type: HookType, f: F)
    where
        F: Fn(&HookContext) + Send + Sync + 'static,
    {
        self.register(hook_type, Arc::new(f));
    }

    pub fn trigger(&self, hook_type: HookType, ctx: &HookContext) {
        let hooks = self.hooks.lock().unwrap();
        if let Some(callbacks) = hooks.get(&hook_type) {
            for cb in callbacks {
                cb(ctx);
            }
        }
    }

    pub fn clear(&self, hook_type: HookType) {
        let mut hooks = self.hooks.lock().unwrap();
        hooks.remove(&hook_type);
    }

    pub fn clear_all(&self) {
        let mut hooks = self.hooks.lock().unwrap();
        hooks.clear();
    }
}
