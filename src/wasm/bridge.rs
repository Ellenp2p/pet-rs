use super::WasmPetPlugin;
use bevy::prelude::*;
use std::sync::{Arc, Mutex};

#[derive(Resource)]
pub struct WasmPluginHost {
    plugins: Arc<Mutex<Vec<Box<dyn WasmPetPlugin>>>>,
}

impl Default for WasmPluginHost {
    fn default() -> Self {
        Self {
            plugins: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl WasmPluginHost {
    pub fn register(&self, plugin: Box<dyn WasmPetPlugin>) {
        let mut plugins = self.plugins.lock().unwrap();
        info!("Registered WASM plugin: {}", plugin.name());
        plugins.push(plugin);
    }

    pub fn trigger_on_tick(&self, pet_id: u64) {
        let plugins = self.plugins.lock().unwrap();
        for plugin in plugins.iter() {
            plugin.on_tick(pet_id);
        }
    }

    pub fn trigger_on_feed(&self, pet_id: u64, amount: f32) {
        let plugins = self.plugins.lock().unwrap();
        for plugin in plugins.iter() {
            plugin.on_feed(pet_id, amount);
        }
    }

    pub fn trigger_on_state_change(&self, pet_id: u64, old_state: &str, new_state: &str) {
        let plugins = self.plugins.lock().unwrap();
        for plugin in plugins.iter() {
            plugin.on_state_change(pet_id, old_state, new_state);
        }
    }

    pub fn trigger_on_network_sync(&self, pet_id: u64) {
        let plugins = self.plugins.lock().unwrap();
        for plugin in plugins.iter() {
            plugin.on_network_sync(pet_id);
        }
    }

    pub fn plugin_count(&self) -> usize {
        self.plugins.lock().unwrap().len()
    }
}
