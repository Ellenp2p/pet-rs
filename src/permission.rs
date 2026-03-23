//! Plugin permission system.
//!
//! This module provides permission control for WASM plugins.
//! Plugins can be granted specific permissions to access resources.

use std::collections::HashSet;

/// Plugin permission types.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Permission {
    /// Read plugin data
    ReadData(String),
    /// Write plugin data
    WriteData(String),
    /// Read configuration
    ReadConfig(String),
    /// Write configuration
    WriteConfig(String),
    /// Access to specific plugin
    AccessPlugin(String),
    /// Full access to all plugins
    FullAccess,
}

/// Plugin permissions configuration.
#[derive(Debug, Clone)]
pub struct PluginPermissions {
    /// Plugin ID
    pub plugin_id: String,
    /// Granted permissions
    pub permissions: HashSet<Permission>,
    /// Denied permissions
    pub denied: HashSet<Permission>,
}

impl PluginPermissions {
    /// Create new permissions for a plugin.
    pub fn new(plugin_id: String) -> Self {
        Self {
            plugin_id,
            permissions: HashSet::new(),
            denied: HashSet::new(),
        }
    }

    /// Grant a permission.
    pub fn grant(&mut self, permission: Permission) {
        self.permissions.insert(permission);
    }

    /// Deny a permission.
    pub fn deny(&mut self, permission: Permission) {
        self.denied.insert(permission);
    }

    /// Check if a permission is granted.
    pub fn has_permission(&self, permission: &Permission) -> bool {
        // Check if explicitly denied
        if self.denied.contains(permission) {
            return false;
        }

        // Check if explicitly granted
        if self.permissions.contains(permission) {
            return true;
        }

        // Check for wildcard permissions
        match permission {
            Permission::ReadData(key) => {
                if self
                    .permissions
                    .contains(&Permission::ReadData("*".to_string()))
                {
                    return !self.denied.contains(&Permission::ReadData(key.clone()));
                }
            }
            Permission::WriteData(key) => {
                if self
                    .permissions
                    .contains(&Permission::WriteData("*".to_string()))
                {
                    return !self.denied.contains(&Permission::WriteData(key.clone()));
                }
            }
            Permission::ReadConfig(key) => {
                if self
                    .permissions
                    .contains(&Permission::ReadConfig("*".to_string()))
                {
                    return !self.denied.contains(&Permission::ReadConfig(key.clone()));
                }
            }
            Permission::WriteConfig(key) => {
                if self
                    .permissions
                    .contains(&Permission::WriteConfig("*".to_string()))
                {
                    return !self.denied.contains(&Permission::WriteConfig(key.clone()));
                }
            }
            Permission::AccessPlugin(plugin_id) => {
                if self
                    .permissions
                    .contains(&Permission::AccessPlugin("*".to_string()))
                {
                    return !self
                        .denied
                        .contains(&Permission::AccessPlugin(plugin_id.clone()));
                }
            }
            Permission::FullAccess => {}
        }

        // Check for full access
        if self.permissions.contains(&Permission::FullAccess) {
            return true;
        }

        false
    }

    /// Check if can read data for a specific key.
    pub fn can_read_data(&self, key: &str) -> bool {
        // First check if wildcard permission is granted and not denied for this key
        if self.has_permission(&Permission::ReadData("*".to_string())) {
            return !self.denied.contains(&Permission::ReadData(key.to_string()));
        }
        // Then check specific key permission
        self.has_permission(&Permission::ReadData(key.to_string()))
    }

    /// Check if can write data for a specific key.
    pub fn can_write_data(&self, key: &str) -> bool {
        if self.has_permission(&Permission::WriteData("*".to_string())) {
            return !self
                .denied
                .contains(&Permission::WriteData(key.to_string()));
        }
        self.has_permission(&Permission::WriteData(key.to_string()))
    }

    /// Check if can read config for a specific key.
    pub fn can_read_config(&self, key: &str) -> bool {
        if self.has_permission(&Permission::ReadConfig("*".to_string())) {
            return !self
                .denied
                .contains(&Permission::ReadConfig(key.to_string()));
        }
        self.has_permission(&Permission::ReadConfig(key.to_string()))
    }

    /// Check if can write config for a specific key.
    pub fn can_write_config(&self, key: &str) -> bool {
        self.has_permission(&Permission::WriteConfig(key.to_string()))
            || self.has_permission(&Permission::WriteConfig("*".to_string()))
    }

    /// Check if can access a specific plugin.
    pub fn can_access_plugin(&self, plugin_id: &str) -> bool {
        self.has_permission(&Permission::AccessPlugin(plugin_id.to_string()))
            || self.has_permission(&Permission::AccessPlugin("*".to_string()))
            || self.has_permission(&Permission::FullAccess)
    }
}

/// Permission manager for plugins.
#[derive(Debug, Clone, Default)]
pub struct PermissionManager {
    /// Plugin permissions indexed by plugin ID
    permissions: std::collections::HashMap<String, PluginPermissions>,
}

impl PermissionManager {
    /// Create a new permission manager.
    pub fn new() -> Self {
        Self {
            permissions: std::collections::HashMap::new(),
        }
    }

    /// Set permissions for a plugin.
    pub fn set_permissions(&mut self, permissions: PluginPermissions) {
        self.permissions
            .insert(permissions.plugin_id.clone(), permissions);
    }

    /// Get permissions for a plugin.
    pub fn get_permissions(&self, plugin_id: &str) -> Option<&PluginPermissions> {
        self.permissions.get(plugin_id)
    }

    /// Check if a plugin has a specific permission.
    pub fn has_permission(&self, plugin_id: &str, permission: &Permission) -> bool {
        if let Some(perms) = self.permissions.get(plugin_id) {
            perms.has_permission(permission)
        } else {
            false
        }
    }

    /// Check if a plugin can read data for a specific key.
    pub fn can_read_data(&self, plugin_id: &str, key: &str) -> bool {
        if let Some(perms) = self.permissions.get(plugin_id) {
            perms.can_read_data(key)
        } else {
            false
        }
    }

    /// Check if a plugin can write data for a specific key.
    pub fn can_write_data(&self, plugin_id: &str, key: &str) -> bool {
        if let Some(perms) = self.permissions.get(plugin_id) {
            perms.can_write_data(key)
        } else {
            false
        }
    }

    /// Check if a plugin can read config for a specific key.
    pub fn can_read_config(&self, plugin_id: &str, key: &str) -> bool {
        if let Some(perms) = self.permissions.get(plugin_id) {
            perms.can_read_config(key)
        } else {
            false
        }
    }

    /// Check if a plugin can write config for a specific key.
    pub fn can_write_config(&self, plugin_id: &str, key: &str) -> bool {
        if let Some(perms) = self.permissions.get(plugin_id) {
            perms.can_write_config(key)
        } else {
            false
        }
    }

    /// Check if a plugin can access another plugin.
    pub fn can_access_plugin(&self, plugin_id: &str, target_plugin_id: &str) -> bool {
        if let Some(perms) = self.permissions.get(plugin_id) {
            perms.can_access_plugin(target_plugin_id)
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_grant_and_check() {
        let mut perms = PluginPermissions::new("test_plugin".to_string());
        perms.grant(Permission::ReadData("key1".to_string()));

        assert!(perms.can_read_data("key1"));
        assert!(!perms.can_read_data("key2"));
    }

    #[test]
    fn test_permission_wildcard() {
        let mut perms = PluginPermissions::new("test_plugin".to_string());
        perms.grant(Permission::ReadData("*".to_string()));

        assert!(perms.can_read_data("any_key"));
        assert!(perms.can_read_data("another_key"));
    }

    #[test]
    fn test_permission_deny() {
        let mut perms = PluginPermissions::new("test_plugin".to_string());
        perms.grant(Permission::ReadData("*".to_string()));
        perms.deny(Permission::ReadData("secret".to_string()));

        assert!(perms.can_read_data("public"));
        assert!(!perms.can_read_data("secret"));
    }

    #[test]
    fn test_permission_full_access() {
        let mut perms = PluginPermissions::new("test_plugin".to_string());
        perms.grant(Permission::FullAccess);

        assert!(perms.can_read_data("any"));
        assert!(perms.can_write_data("any"));
        assert!(perms.can_read_config("any"));
        assert!(perms.can_access_plugin("other_plugin"));
    }

    #[test]
    fn test_permission_manager() {
        let mut manager = PermissionManager::new();

        let mut perms = PluginPermissions::new("plugin_a".to_string());
        perms.grant(Permission::ReadData("*".to_string()));
        manager.set_permissions(perms);

        assert!(manager.can_read_data("plugin_a", "key1"));
        assert!(!manager.can_read_data("plugin_b", "key1"));
    }
}
