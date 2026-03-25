//! 插件发现
//!
//! 负责发现和扫描插件。

/// 插件发现器
pub struct PluginDiscovery {
    /// 搜索路径
    search_paths: Vec<std::path::PathBuf>,
}

impl PluginDiscovery {
    /// 创建新的插件发现器
    pub fn new() -> Self {
        Self {
            search_paths: Vec::new(),
        }
    }

    /// 添加搜索路径
    pub fn add_search_path(&mut self, path: impl Into<std::path::PathBuf>) {
        self.search_paths.push(path.into());
    }

    /// 发现插件
    #[cfg(feature = "wasm-plugin")]
    pub fn discover(&self) -> Result<Vec<DiscoveredPlugin>, FrameworkError> {
        let mut plugins = Vec::new();

        for path in &self.search_paths {
            if path.is_dir() {
                let manifest_path = path.join("manifest.json");
                if manifest_path.exists() {
                    match PluginManifestLoader::load_from_file(&manifest_path) {
                        Ok(manifest) => {
                            plugins.push(DiscoveredPlugin {
                                path: path.clone(),
                                manifest,
                            });
                        }
                        Err(e) => {
                            log::warn!("Failed to load manifest from {:?}: {}", path, e);
                        }
                    }
                }
            }
        }

        Ok(plugins)
    }

    /// 获取搜索路径
    pub fn search_paths(&self) -> &[std::path::PathBuf] {
        &self.search_paths
    }
}

impl Default for PluginDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

/// 发现的插件
#[derive(Debug, Clone)]
pub struct DiscoveredPlugin {
    /// 插件路径
    pub path: std::path::PathBuf,
    /// 插件 Manifest
    #[cfg(feature = "wasm-plugin")]
    pub manifest: crate::wasm::abi::PluginManifest,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_discovery() {
        let mut discovery = PluginDiscovery::new();
        discovery.add_search_path("/path/to/plugins");
        assert_eq!(discovery.search_paths().len(), 1);
    }
}
