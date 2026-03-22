use bevy::prelude::*;

#[cfg(feature = "wasm-plugin")]
use {
    super::{wasmtime_loader::WasmtimePlugin, WasmPlugin, WasmPluginId},
    crate::error::FrameworkError,
    std::collections::HashMap,
    std::sync::{Arc, Mutex},
};

/// Host-side container for registered WASM plugins.
///
/// Supports hot reload: plugins with same ID are replaced, different IDs coexist.
#[derive(Resource, Default)]
pub struct WasmPluginHost {
    #[cfg(feature = "wasm-plugin")]
    plugins: Arc<Mutex<Vec<Box<dyn WasmPlugin>>>>,
    #[cfg(feature = "wasm-plugin")]
    plugin_states: Arc<Mutex<HashMap<WasmPluginId, Vec<u8>>>>,
}

impl WasmPluginHost {
    /// Register a WASM plugin from a .wasm file.
    ///
    /// ## Hot Reload Behavior
    ///
    /// - If `plugin_id` is None, uses the wasm internal name
    /// - If `plugin_id` already exists, the old plugin is replaced
    /// - If `plugin_id` is new, the plugin is added to the list
    ///
    /// ## Example
    ///
    /// ```ignore
    /// host.register_wasm(std::path::Path::new("plugin.wasm"), None)?;
    /// host.register_wasm(std::path::Path::new("plugin_v2.wasm"), Some("my_plugin".into()))?;
    /// ```
    #[cfg(feature = "wasm-plugin")]
    pub fn register_wasm(
        &self,
        path: &std::path::Path,
        plugin_id: Option<String>,
    ) -> Result<(), FrameworkError> {
        // Load the new plugin
        let new_plugin = WasmtimePlugin::load(path, plugin_id.clone())?;
        let new_id = new_plugin.id().as_str().to_string();

        let mut plugins = self
            .plugins
            .lock()
            .map_err(|_| FrameworkError::LockPoisoned)?;

        // Check if plugin with same ID exists (replace)
        if let Some(pos) = plugins.iter().position(|p| p.id().as_str() == new_id) {
            info!("WASM plugin '{}' replaced (hot reload)", new_id);
            plugins[pos] = Box::new(new_plugin);
        } else {
            info!("WASM plugin '{}' loaded", new_id);
            plugins.push(Box::new(new_plugin));
            // Initialize empty state for new plugin
            let mut states = self
                .plugin_states
                .lock()
                .map_err(|_| FrameworkError::LockPoisoned)?;
            states
                .entry(WasmPluginId::new(new_id.clone()))
                .or_insert_with(Vec::new);
        }

        Ok(())
    }

    #[cfg(not(feature = "wasm-plugin"))]
    pub fn register_wasm(
        &self,
        _path: &std::path::Path,
        _plugin_id: Option<String>,
    ) -> Result<(), FrameworkError> {
        Ok(())
    }

    /// Unregister a WASM plugin by ID.
    ///
    /// Returns error if plugin not found.
    #[cfg(feature = "wasm-plugin")]
    pub fn unregister_wasm(&self, plugin_id: &str) -> Result<(), FrameworkError> {
        let mut plugins = self
            .plugins
            .lock()
            .map_err(|_| FrameworkError::LockPoisoned)?;

        let pos = plugins
            .iter()
            .position(|p| p.id().as_str() == plugin_id)
            .ok_or_else(|| {
                FrameworkError::WasmUnload(format!("plugin not found: {}", plugin_id))
            })?;

        let removed = plugins.remove(pos);
        info!("WASM plugin '{}' unloaded", removed.name());
        Ok(())
    }

    #[cfg(not(feature = "wasm-plugin"))]
    pub fn unregister_wasm(&self, _plugin_id: &str) -> Result<(), FrameworkError> {
        Ok(())
    }

    /// Get count of registered plugins.
    #[cfg(feature = "wasm-plugin")]
    pub fn plugin_count(&self) -> Result<usize, FrameworkError> {
        let plugins = self
            .plugins
            .lock()
            .map_err(|_| FrameworkError::LockPoisoned)?;
        Ok(plugins.len())
    }

    #[cfg(not(feature = "wasm-plugin"))]
    pub fn plugin_count(&self) -> Result<usize, FrameworkError> {
        Ok(0)
    }

    /// Trigger on_tick for all plugins.
    #[cfg(feature = "wasm-plugin")]
    pub fn trigger_on_tick(&self, entity_id: u64) -> Result<(), FrameworkError> {
        let plugins = self
            .plugins
            .lock()
            .map_err(|_| FrameworkError::LockPoisoned)?;

        for plugin in plugins.iter() {
            plugin.on_tick(super::WasmEntityId(entity_id));
        }
        Ok(())
    }

    #[cfg(not(feature = "wasm-plugin"))]
    pub fn trigger_on_tick(&self, _entity_id: u64) -> Result<(), FrameworkError> {
        Ok(())
    }

    /// Trigger on_event for all plugins.
    #[cfg(feature = "wasm-plugin")]
    pub fn trigger_on_event(
        &self,
        entity_id: u64,
        event: &str,
        data: &str,
    ) -> Result<(), FrameworkError> {
        let plugins = self
            .plugins
            .lock()
            .map_err(|_| FrameworkError::LockPoisoned)?;

        for plugin in plugins.iter() {
            plugin.on_event(super::WasmEntityId(entity_id), event, data);

            // Sync plugin state to host after event processing
            if let Some(state) = plugin.get_state() {
                let plugin_id = plugin.id().clone();
                let mut states = self
                    .plugin_states
                    .lock()
                    .map_err(|_| FrameworkError::LockPoisoned)?;
                states.insert(plugin_id, state);
            }
        }
        Ok(())
    }

    #[cfg(not(feature = "wasm-plugin"))]
    pub fn trigger_on_event(
        &self,
        _entity_id: u64,
        _event: &str,
        _data: &str,
    ) -> Result<(), FrameworkError> {
        Ok(())
    }

    /// Get plugin state by ID.
    #[cfg(feature = "wasm-plugin")]
    pub fn get_plugin_state(&self, plugin_id: &str) -> Result<Option<Vec<u8>>, FrameworkError> {
        let states = self
            .plugin_states
            .lock()
            .map_err(|_| FrameworkError::LockPoisoned)?;
        Ok(states.get(&WasmPluginId::new(plugin_id)).cloned())
    }

    #[cfg(not(feature = "wasm-plugin"))]
    pub fn get_plugin_state(&self, _plugin_id: &str) -> Result<Option<Vec<u8>>, FrameworkError> {
        Ok(None)
    }

    /// Set plugin state by ID.
    #[cfg(feature = "wasm-plugin")]
    pub fn set_plugin_state(&self, plugin_id: &str, state: Vec<u8>) -> Result<(), FrameworkError> {
        let mut states = self
            .plugin_states
            .lock()
            .map_err(|_| FrameworkError::LockPoisoned)?;
        states.insert(WasmPluginId::new(plugin_id), state);
        Ok(())
    }

    #[cfg(not(feature = "wasm-plugin"))]
    pub fn set_plugin_state(
        &self,
        _plugin_id: &str,
        _state: Vec<u8>,
    ) -> Result<(), FrameworkError> {
        Ok(())
    }

    /// Remove plugin state by ID.
    #[cfg(feature = "wasm-plugin")]
    pub fn remove_plugin_state(&self, plugin_id: &str) -> Result<(), FrameworkError> {
        let mut states = self
            .plugin_states
            .lock()
            .map_err(|_| FrameworkError::LockPoisoned)?;
        states.remove(&WasmPluginId::new(plugin_id));
        Ok(())
    }

    #[cfg(not(feature = "wasm-plugin"))]
    pub fn remove_plugin_state(&self, _plugin_id: &str) -> Result<(), FrameworkError> {
        Ok(())
    }
}
