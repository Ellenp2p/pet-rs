use bevy::prelude::*;

#[cfg(feature = "wasm-plugin")]
use {
    super::{wasmtime_loader::WasmtimePlugin, WasmPlugin, WasmPluginId},
    crate::config::PluginConfigManager,
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
    #[cfg(feature = "wasm-plugin")]
    plugin_data: Arc<Mutex<HashMap<WasmPluginId, HashMap<String, Vec<u8>>>>>,
    #[cfg(feature = "wasm-plugin")]
    config_manager: Option<Arc<Mutex<PluginConfigManager>>>,
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
        // Create callbacks for inter-plugin communication
        let plugin_data_clone = self.plugin_data.clone();
        let read_data_fn: Arc<dyn Fn(&str, &str) -> Option<Vec<u8>> + Send + Sync> =
            Arc::new(move |_plugin_id: &str, key: &str| {
                // For now, we read from the current plugin's data
                // In a full implementation, we'd need to pass the target plugin ID
                let data = plugin_data_clone.lock().ok()?;
                // Try to find any plugin that has this key
                for (_, plugin_data) in data.iter() {
                    if let Some(value) = plugin_data.get(key) {
                        return Some(value.clone());
                    }
                }
                None
            });

        let plugin_data_clone2 = self.plugin_data.clone();
        let write_data_fn: Arc<dyn Fn(&str, &str, Vec<u8>) + Send + Sync> =
            Arc::new(move |_plugin_id: &str, key: &str, value: Vec<u8>| {
                // For now, we write to all plugins' data
                // In a full implementation, we'd need to pass the target plugin ID
                if let Ok(mut data) = plugin_data_clone2.lock() {
                    // Write to all plugins (simplified approach)
                    for (_, plugin_data) in data.iter_mut() {
                        plugin_data.insert(key.to_string(), value.clone());
                    }
                }
            });

        // Create config read callback
        let config_manager_clone = self.config_manager.clone();
        let read_config_fn: Arc<dyn Fn(&str, &str) -> Option<String> + Send + Sync> =
            Arc::new(move |plugin_id: &str, key: &str| {
                if let Some(config_manager) = &config_manager_clone {
                    if let Ok(manager) = config_manager.lock() {
                        if let Ok(Some(value)) = manager.get_config(plugin_id, key) {
                            return Some(format!("{:?}", value));
                        }
                    }
                }
                None
            });

        // Load the new plugin with callbacks
        let new_plugin = WasmtimePlugin::load_with_callbacks(
            path,
            plugin_id.clone(),
            read_data_fn,
            write_data_fn,
            read_config_fn,
        )?;
        let new_id = new_plugin.id().as_str().to_string();

        let mut plugins = self
            .plugins
            .lock()
            .map_err(|_| FrameworkError::LockPoisoned)?;

        // Check if plugin with same ID exists (replace)
        if let Some(pos) = plugins.iter().position(|p| p.id().as_str() == new_id) {
            // Call on_unload for the old plugin before replacing
            if let Err(e) = plugins[pos].on_unload() {
                log::warn!("Failed to call on_unload for plugin '{}': {}", new_id, e);
            }
            info!("WASM plugin '{}' replaced (hot reload)", new_id);
            plugins[pos] = Box::new(new_plugin);
            // Call on_load for the new plugin
            if let Err(e) = plugins[pos].on_load() {
                log::error!("Failed to call on_load for plugin '{}': {}", new_id, e);
            }
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

            // Initialize empty data storage for new plugin
            let mut data = self
                .plugin_data
                .lock()
                .map_err(|_| FrameworkError::LockPoisoned)?;
            data.entry(WasmPluginId::new(new_id.clone()))
                .or_insert_with(HashMap::new);

            // Call on_load for the new plugin
            if let Some(plugin) = plugins.last() {
                if let Err(e) = plugin.on_load() {
                    log::error!("Failed to call on_load for plugin '{}': {}", new_id, e);
                }
            }
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

    /// Set the configuration manager for plugins.
    #[cfg(feature = "wasm-plugin")]
    pub fn set_config_manager(&mut self, config_manager: PluginConfigManager) {
        self.config_manager = Some(Arc::new(Mutex::new(config_manager)));
    }

    #[cfg(not(feature = "wasm-plugin"))]
    pub fn set_config_manager(&mut self, _config_manager: crate::config::PluginConfigManager) {
        // Stub implementation
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

        // Call on_unload before removing the plugin
        if let Err(e) = plugins[pos].on_unload() {
            log::warn!("Failed to call on_unload for plugin '{}': {}", plugin_id, e);
        }

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

        // Check if this is a plugin data request
        if event.starts_with("plugin_request:") {
            return self.handle_plugin_request(entity_id, event, data, &plugins);
        }

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

    /// Handle plugin data request events
    #[cfg(feature = "wasm-plugin")]
    fn handle_plugin_request(
        &self,
        entity_id: u64,
        event: &str,
        _data: &str,
        plugins: &Vec<Box<dyn WasmPlugin>>,
    ) -> Result<(), FrameworkError> {
        // Parse request: "plugin_request:target_plugin:data_key"
        let parts: Vec<&str> = event.split(':').collect();
        if parts.len() < 3 {
            return Ok(());
        }

        let target_plugin_id = parts[1];
        let data_key = parts[2];

        // Read specific data from target plugin using read_plugin_data
        if let Ok(Some(data_value)) = self.read_plugin_data(target_plugin_id, data_key) {
            // Convert data to hex string for transmission
            let hex_data: String = data_value.iter().map(|b| format!("{:02x}", b)).collect();

            // Send response event back to requesting plugin
            let response_event = format!("plugin_response:{}:{}", target_plugin_id, data_key);

            // Trigger response event for all plugins (the requesting plugin will handle it)
            for plugin in plugins.iter() {
                plugin.on_event(super::WasmEntityId(entity_id), &response_event, &hex_data);
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

    /// Read data from another plugin (inter-plugin communication).
    #[cfg(feature = "wasm-plugin")]
    pub fn read_plugin_data(
        &self,
        source_plugin_id: &str,
        data_key: &str,
    ) -> Result<Option<Vec<u8>>, FrameworkError> {
        let data = self
            .plugin_data
            .lock()
            .map_err(|_| FrameworkError::LockPoisoned)?;

        let source_id = WasmPluginId::new(source_plugin_id);
        if let Some(plugin_data) = data.get(&source_id) {
            Ok(plugin_data.get(data_key).cloned())
        } else {
            Ok(None)
        }
    }

    #[cfg(not(feature = "wasm-plugin"))]
    pub fn read_plugin_data(
        &self,
        _source_plugin_id: &str,
        _data_key: &str,
    ) -> Result<Option<Vec<u8>>, FrameworkError> {
        Ok(None)
    }

    /// Write data to plugin (for inter-plugin communication).
    #[cfg(feature = "wasm-plugin")]
    pub fn write_plugin_data(
        &self,
        plugin_id: &str,
        data_key: &str,
        data: Vec<u8>,
    ) -> Result<(), FrameworkError> {
        let mut plugin_data = self
            .plugin_data
            .lock()
            .map_err(|_| FrameworkError::LockPoisoned)?;

        let id = WasmPluginId::new(plugin_id);
        plugin_data
            .entry(id)
            .or_insert_with(HashMap::new)
            .insert(data_key.to_string(), data);

        Ok(())
    }

    #[cfg(not(feature = "wasm-plugin"))]
    pub fn write_plugin_data(
        &self,
        _plugin_id: &str,
        _data_key: &str,
        _data: Vec<u8>,
    ) -> Result<(), FrameworkError> {
        Ok(())
    }
}
