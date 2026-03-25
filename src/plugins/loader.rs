//! 插件加载器
//!
//! 负责加载 WASM 插件。

use crate::error::FrameworkError;

/// 插件加载器
pub struct PluginLoader {
    /// 插件路径
    plugin_paths: Vec<std::path::PathBuf>,
}

impl PluginLoader {
    /// 创建新的插件加载器
    pub fn new() -> Self {
        Self {
            plugin_paths: Vec::new(),
        }
    }

    /// 添加插件路径
    pub fn add_path(&mut self, path: impl Into<std::path::PathBuf>) {
        self.plugin_paths.push(path.into());
    }

    /// 获取插件路径
    pub fn paths(&self) -> &[std::path::PathBuf] {
        &self.plugin_paths
    }

    /// 加载插件
    #[cfg(feature = "wasm-plugin")]
    pub fn load(&self, manifest: &crate::wasm::abi::PluginManifest) -> Result<(), FrameworkError> {
        log::info!("Loading plugin: {} v{}", manifest.name, manifest.version);
        Ok(())
    }

    /// 卸载插件
    pub fn unload(&self, plugin_id: &str) -> Result<(), FrameworkError> {
        log::info!("Unloading plugin: {}", plugin_id);
        Ok(())
    }
}

impl Default for PluginLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_loader() {
        let mut loader = PluginLoader::new();
        loader.add_path("/path/to/plugins");
        assert_eq!(loader.paths().len(), 1);
    }
}
