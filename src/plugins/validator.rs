//! 插件验证器
//!
//! 负责验证插件的完整性和安全性。

#[cfg(feature = "wasm-plugin")]
use crate::error::FrameworkError;

/// 插件验证器
pub struct PluginValidator {
    /// 是否启用安全检查
    security_check: bool,
}

impl PluginValidator {
    /// 创建新的插件验证器
    pub fn new() -> Self {
        Self {
            security_check: true,
        }
    }

    /// 设置是否启用安全检查
    pub fn set_security_check(&mut self, enabled: bool) {
        self.security_check = enabled;
    }

    /// 验证插件
    #[cfg(feature = "wasm-plugin")]
    pub fn validate(
        &self,
        manifest: &crate::wasm::abi::PluginManifest,
    ) -> Result<ValidationResult, FrameworkError> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        // 基础验证
        if manifest.name.is_empty() {
            errors.push("Plugin name is required".to_string());
        }

        if manifest.version.is_empty() {
            errors.push("Plugin version is required".to_string());
        }

        if manifest.wasm_entry.is_empty() {
            errors.push("WASM entry file is required".to_string());
        }

        // 安全检查
        if self.security_check {
            // 检查权限
            for permission in &manifest.permissions {
                if permission.denied {
                    warnings.push(format!(
                        "Permission denied for resource: {}",
                        permission.resource
                    ));
                }
            }

            // 检查网络权限
            let has_network = manifest.permissions.iter().any(|p| {
                matches!(
                    p.permission_type,
                    crate::wasm::abi::PermissionType::NetworkAccess
                )
            });
            if has_network {
                warnings.push("Plugin requests network access".to_string());
            }
        }

        // 依赖检查
        for dep in &manifest.dependencies {
            if dep.version_req.is_empty() {
                warnings.push(format!(
                    "Dependency '{}' has no version requirement",
                    dep.plugin_id
                ));
            }
        }

        let is_valid = errors.is_empty();

        Ok(ValidationResult {
            is_valid,
            warnings,
            errors,
        })
    }
}

impl Default for PluginValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// 验证结果
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// 是否有效
    pub is_valid: bool,
    /// 警告信息
    pub warnings: Vec<String>,
    /// 错误信息
    pub errors: Vec<String>,
}

impl ValidationResult {
    /// 是否有警告
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    /// 是否有错误
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// 获取所有问题
    pub fn all_issues(&self) -> Vec<String> {
        let mut issues = self.errors.clone();
        issues.extend(self.warnings.clone());
        issues
    }
}

#[cfg(test)]
#[cfg(feature = "wasm-plugin")]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_validator() {
        let validator = PluginValidator::new();
        let manifest =
            crate::wasm::abi::PluginManifest::new("test", "1.0.0").with_wasm_entry("test.wasm");

        let result = validator.validate(&manifest).unwrap();
        assert!(result.is_valid);
    }

    #[cfg(feature = "wasm-plugin")]
    #[test]
    fn test_plugin_validator_errors() {
        let validator = PluginValidator::new();
        let manifest = crate::wasm::abi::PluginManifest::new("", "");

        let result = validator.validate(&manifest).unwrap();
        assert!(!result.is_valid);
        assert!(result.has_errors());
    }
}
