//! Manifest 加载器
//!
//! 负责加载和解析插件 Manifest 文件。

#[cfg(feature = "wasm-plugin")]
use crate::error::FrameworkError;
#[cfg(feature = "wasm-plugin")]
use std::path::Path;

/// Manifest 加载器
pub struct PluginManifestLoader;

impl PluginManifestLoader {
    /// 从文件加载 Manifest
    #[cfg(feature = "wasm-plugin")]
    pub fn load_from_file(path: &Path) -> Result<crate::wasm::abi::PluginManifest, FrameworkError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| FrameworkError::Other(format!("Failed to read manifest file: {}", e)))?;

        let manifest: crate::wasm::abi::PluginManifest = serde_json::from_str(&content)
            .map_err(|e| FrameworkError::Other(format!("Failed to parse manifest: {}", e)))?;

        manifest
            .validate()
            .map_err(|e| FrameworkError::Other(format!("Invalid manifest: {}", e)))?;

        Ok(manifest)
    }

    /// 从 JSON 字符串加载 Manifest
    #[cfg(feature = "wasm-plugin")]
    pub fn load_from_json(json: &str) -> Result<crate::wasm::abi::PluginManifest, FrameworkError> {
        let manifest: crate::wasm::abi::PluginManifest = serde_json::from_str(json)
            .map_err(|e| FrameworkError::Other(format!("Failed to parse manifest: {}", e)))?;

        manifest
            .validate()
            .map_err(|e| FrameworkError::Other(format!("Invalid manifest: {}", e)))?;

        Ok(manifest)
    }

    /// 保存 Manifest 到文件
    #[cfg(feature = "wasm-plugin")]
    pub fn save_to_file(
        manifest: &crate::wasm::abi::PluginManifest,
        path: &Path,
    ) -> Result<(), FrameworkError> {
        let content = serde_json::to_string_pretty(manifest)
            .map_err(|e| FrameworkError::Other(format!("Failed to serialize manifest: {}", e)))?;

        std::fs::write(path, content)
            .map_err(|e| FrameworkError::Other(format!("Failed to write manifest file: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
#[cfg(feature = "wasm-plugin")]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_from_json() {
        let json = r#"
        {
            "name": "test-plugin",
            "version": "1.0.0",
            "description": "A test plugin",
            "author": "Test",
            "license": "MIT",
            "plugin_type": "Hook",
            "capabilities": ["test"],
            "hooks": ["on_input_received"],
            "slots": [],
            "permissions": [],
            "dependencies": [],
            "config_schema": {},
            "wasm_entry": "test.wasm"
        }
        "#;

        let manifest = PluginManifestLoader::load_from_json(json).unwrap();
        assert_eq!(manifest.name, "test-plugin");
        assert_eq!(manifest.version, "1.0.0");
    }
}
