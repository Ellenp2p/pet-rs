//! Plugin configuration system.
//!
//! This module provides configuration management for WASM plugins.
//! Plugins can read configuration values from JSON files.
//!
//! ## Configuration Format
//!
//! Configuration files are stored in JSON format with the following structure:
//! ```json
//! {
//!   "plugin_id": {
//!     "setting_key": "value",
//!     "numeric_setting": 42,
//!     "boolean_setting": true
//!   }
//! }
//! ```

use bevy::prelude::*;
use std::collections::HashMap;
use std::path::Path;

#[cfg(feature = "wasm-plugin")]
use {
    crate::error::FrameworkError,
    std::fs,
    std::sync::{Arc, Mutex},
};

/// Plugin configuration value types.
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigValue {
    /// String value
    String(String),
    /// Numeric value
    Number(f64),
    /// Boolean value
    Bool(bool),
    /// Array of values
    Array(Vec<ConfigValue>),
    /// Object (nested configuration)
    Object(HashMap<String, ConfigValue>),
}

impl ConfigValue {
    /// Get string value if this is a string.
    pub fn as_string(&self) -> Option<&str> {
        match self {
            ConfigValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// Get number value if this is a number.
    pub fn as_number(&self) -> Option<f64> {
        match self {
            ConfigValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    /// Get boolean value if this is a boolean.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            ConfigValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Get array value if this is an array.
    pub fn as_array(&self) -> Option<&Vec<ConfigValue>> {
        match self {
            ConfigValue::Array(arr) => Some(arr),
            _ => None,
        }
    }

    /// Get object value if this is an object.
    pub fn as_object(&self) -> Option<&HashMap<String, ConfigValue>> {
        match self {
            ConfigValue::Object(obj) => Some(obj),
            _ => None,
        }
    }
}

/// Plugin configuration.
#[derive(Debug, Clone)]
pub struct PluginConfig {
    /// Plugin ID
    pub plugin_id: String,
    /// Configuration values
    pub values: HashMap<String, ConfigValue>,
}

/// Configuration manager for plugins.
///
/// Manages configuration files for WASM plugins.
/// Configuration files are stored in JSON format.
#[cfg(feature = "wasm-plugin")]
#[derive(Resource, Default)]
pub struct PluginConfigManager {
    /// Plugin configurations indexed by plugin ID
    configs: Arc<Mutex<HashMap<String, PluginConfig>>>,
}

#[cfg(feature = "wasm-plugin")]
impl PluginConfigManager {
    /// Load configuration from a JSON file.
    ///
    /// ## Arguments
    ///
    /// * `path` - Path to the JSON configuration file
    ///
    /// ## Returns
    ///
    /// Returns `Ok(())` if configuration was loaded successfully.
    /// Returns `Err` if the file cannot be read or parsed.
    pub fn load_from_file(&self, path: &Path) -> Result<(), FrameworkError> {
        let content = fs::read_to_string(path)
            .map_err(|e| FrameworkError::Plugin(format!("Failed to read config file: {}", e)))?;

        let json: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| FrameworkError::Plugin(format!("Failed to parse config JSON: {}", e)))?;

        let mut configs = self
            .configs
            .lock()
            .map_err(|_| FrameworkError::LockPoisoned)?;

        // Parse JSON into plugin configurations
        if let Some(obj) = json.as_object() {
            for (plugin_id, config_value) in obj {
                let plugin_config = Self::parse_config_value(config_value);
                configs.insert(
                    plugin_id.clone(),
                    PluginConfig {
                        plugin_id: plugin_id.clone(),
                        values: plugin_config,
                    },
                );
            }
        }

        Ok(())
    }

    /// Parse a JSON value into a HashMap of ConfigValue.
    fn parse_config_value(value: &serde_json::Value) -> HashMap<String, ConfigValue> {
        let mut result = HashMap::new();

        if let Some(obj) = value.as_object() {
            for (key, val) in obj {
                let config_val = Self::parse_json_value(val);
                result.insert(key.clone(), config_val);
            }
        }

        result
    }

    /// Parse a single JSON value into ConfigValue.
    fn parse_json_value(value: &serde_json::Value) -> ConfigValue {
        match value {
            serde_json::Value::String(s) => ConfigValue::String(s.clone()),
            serde_json::Value::Number(n) => ConfigValue::Number(n.as_f64().unwrap_or(0.0)),
            serde_json::Value::Bool(b) => ConfigValue::Bool(*b),
            serde_json::Value::Array(arr) => {
                let values: Vec<ConfigValue> = arr.iter().map(Self::parse_json_value).collect();
                ConfigValue::Array(values)
            }
            serde_json::Value::Object(obj) => {
                let mut map = HashMap::new();
                for (key, val) in obj {
                    map.insert(key.clone(), Self::parse_json_value(val));
                }
                ConfigValue::Object(map)
            }
            serde_json::Value::Null => ConfigValue::String(String::new()),
        }
    }

    /// Get configuration value for a plugin.
    ///
    /// ## Arguments
    ///
    /// * `plugin_id` - The plugin ID
    /// * `key` - The configuration key
    ///
    /// ## Returns
    ///
    /// Returns `Some(ConfigValue)` if the configuration exists.
    /// Returns `None` if the plugin or key doesn't exist.
    pub fn get_config(
        &self,
        plugin_id: &str,
        key: &str,
    ) -> Result<Option<ConfigValue>, FrameworkError> {
        let configs = self
            .configs
            .lock()
            .map_err(|_| FrameworkError::LockPoisoned)?;

        if let Some(plugin_config) = configs.get(plugin_id) {
            Ok(plugin_config.values.get(key).cloned())
        } else {
            Ok(None)
        }
    }

    /// Set configuration value for a plugin.
    ///
    /// ## Arguments
    ///
    /// * `plugin_id` - The plugin ID
    /// * `key` - The configuration key
    /// * `value` - The configuration value
    ///
    /// ## Returns
    ///
    /// Returns `Ok(())` if configuration was set successfully.
    pub fn set_config(
        &self,
        plugin_id: &str,
        key: &str,
        value: ConfigValue,
    ) -> Result<(), FrameworkError> {
        let mut configs = self
            .configs
            .lock()
            .map_err(|_| FrameworkError::LockPoisoned)?;

        let plugin_config = configs
            .entry(plugin_id.to_string())
            .or_insert_with(|| PluginConfig {
                plugin_id: plugin_id.to_string(),
                values: HashMap::new(),
            });

        plugin_config.values.insert(key.to_string(), value);
        Ok(())
    }

    /// Check if configuration exists for a plugin.
    pub fn has_config(&self, plugin_id: &str, key: &str) -> Result<bool, FrameworkError> {
        let configs = self
            .configs
            .lock()
            .map_err(|_| FrameworkError::LockPoisoned)?;

        if let Some(plugin_config) = configs.get(plugin_id) {
            Ok(plugin_config.values.contains_key(key))
        } else {
            Ok(false)
        }
    }

    /// Get all configuration keys for a plugin.
    pub fn get_config_keys(&self, plugin_id: &str) -> Result<Vec<String>, FrameworkError> {
        let configs = self
            .configs
            .lock()
            .map_err(|_| FrameworkError::LockPoisoned)?;

        if let Some(plugin_config) = configs.get(plugin_id) {
            Ok(plugin_config.values.keys().cloned().collect())
        } else {
            Ok(Vec::new())
        }
    }
}

/// Stub implementation for non-wasm-plugin feature.
#[cfg(not(feature = "wasm-plugin"))]
#[derive(Resource, Default)]
pub struct PluginConfigManager;

#[cfg(not(feature = "wasm-plugin"))]
impl PluginConfigManager {
    /// Load configuration from a JSON file (stub).
    pub fn load_from_file(&self, _path: &Path) -> Result<(), crate::error::FrameworkError> {
        Ok(())
    }

    /// Get configuration value for a plugin (stub).
    pub fn get_config(
        &self,
        _plugin_id: &str,
        _key: &str,
    ) -> Result<Option<ConfigValue>, crate::error::FrameworkError> {
        Ok(None)
    }

    /// Set configuration value for a plugin (stub).
    pub fn set_config(
        &self,
        _plugin_id: &str,
        _key: &str,
        _value: ConfigValue,
    ) -> Result<(), crate::error::FrameworkError> {
        Ok(())
    }

    /// Check if configuration exists for a plugin (stub).
    pub fn has_config(
        &self,
        _plugin_id: &str,
        _key: &str,
    ) -> Result<bool, crate::error::FrameworkError> {
        Ok(false)
    }

    /// Get all configuration keys for a plugin (stub).
    pub fn get_config_keys(
        &self,
        _plugin_id: &str,
    ) -> Result<Vec<String>, crate::error::FrameworkError> {
        Ok(Vec::new())
    }
}
