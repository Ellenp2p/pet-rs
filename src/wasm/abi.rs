//! WASM ABI 定义
//!
//! 定义 WASM 插件与宿主之间的接口。

use serde::{Deserialize, Serialize};

/// 插件类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PluginType {
    /// 能力插件（工具、技能）
    Capability,
    /// Hook 插件（拦截、修改）
    Hook,
    /// 提供者插件（LLM、记忆）
    Provider,
    /// 通道插件（通信、API）
    Channel,
}

/// 插件权限
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginPermission {
    /// 权限类型
    pub permission_type: PermissionType,
    /// 资源
    pub resource: String,
    /// 是否拒绝
    pub denied: bool,
}

/// 权限类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PermissionType {
    /// 网络访问
    NetworkAccess,
    /// 文件读取
    FileRead,
    /// 文件写入
    FileWrite,
    /// 工具调用
    ToolCall,
    /// 插件调用
    PluginCall,
    /// 自定义
    Custom(String),
}

/// 插件依赖
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDependency {
    /// 插件 ID
    pub plugin_id: String,
    /// 版本要求
    pub version_req: String,
}

/// 插件配置 Schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSchema {
    /// 字段类型
    pub field_type: ConfigFieldType,
    /// 是否必填
    pub required: bool,
    /// 默认值
    pub default: Option<serde_json::Value>,
}

/// 配置字段类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigFieldType {
    /// 字符串
    String,
    /// 整数
    Integer,
    /// 浮点数
    Float,
    /// 布尔值
    Boolean,
    /// 数组
    Array(Box<ConfigFieldType>),
    /// 对象
    Object(std::collections::HashMap<String, ConfigSchema>),
}

/// 插件 Manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    /// 插件名称
    pub name: String,
    /// 版本
    pub version: String,
    /// 描述
    pub description: String,
    /// 作者
    pub author: String,
    /// 许可证
    pub license: String,
    /// 插件类型
    pub plugin_type: PluginType,
    /// 能力列表
    pub capabilities: Vec<String>,
    /// Hook 列表
    pub hooks: Vec<String>,
    /// Slot 列表
    pub slots: Vec<String>,
    /// 权限列表
    pub permissions: Vec<PluginPermission>,
    /// 依赖列表
    pub dependencies: Vec<PluginDependency>,
    /// 配置 Schema
    pub config_schema: std::collections::HashMap<String, ConfigSchema>,
    /// WASM 入口文件
    pub wasm_entry: String,
}

impl PluginManifest {
    /// 创建新的 Manifest
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            description: String::new(),
            author: String::new(),
            license: "MIT".to_string(),
            plugin_type: PluginType::Capability,
            capabilities: Vec::new(),
            hooks: Vec::new(),
            slots: Vec::new(),
            permissions: Vec::new(),
            dependencies: Vec::new(),
            config_schema: std::collections::HashMap::new(),
            wasm_entry: String::new(),
        }
    }

    /// 设置描述
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// 设置作者
    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = author.into();
        self
    }

    /// 设置插件类型
    pub fn with_plugin_type(mut self, plugin_type: PluginType) -> Self {
        self.plugin_type = plugin_type;
        self
    }

    /// 添加能力
    pub fn with_capability(mut self, capability: impl Into<String>) -> Self {
        self.capabilities.push(capability.into());
        self
    }

    /// 添加 Hook
    pub fn with_hook(mut self, hook: impl Into<String>) -> Self {
        self.hooks.push(hook.into());
        self
    }

    /// 添加 Slot
    pub fn with_slot(mut self, slot: impl Into<String>) -> Self {
        self.slots.push(slot.into());
        self
    }

    /// 添加权限
    pub fn with_permission(mut self, permission: PluginPermission) -> Self {
        self.permissions.push(permission);
        self
    }

    /// 添加依赖
    pub fn with_dependency(mut self, dependency: PluginDependency) -> Self {
        self.dependencies.push(dependency);
        self
    }

    /// 设置 WASM 入口
    pub fn with_wasm_entry(mut self, wasm_entry: impl Into<String>) -> Self {
        self.wasm_entry = wasm_entry.into();
        self
    }

    /// 验证 Manifest
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Plugin name is required".to_string());
        }
        if self.version.is_empty() {
            return Err("Plugin version is required".to_string());
        }
        if self.wasm_entry.is_empty() {
            return Err("WASM entry file is required".to_string());
        }
        Ok(())
    }
}

/// Hook 调用上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookCallContext {
    /// Hook 名称
    pub hook_name: String,
    /// Agent ID
    pub agent_id: String,
    /// 会话 ID
    pub session_id: Option<String>,
    /// 输入数据
    pub input: Option<serde_json::Value>,
    /// 输出数据
    pub output: Option<serde_json::Value>,
    /// 上下文数据
    pub data: std::collections::HashMap<String, serde_json::Value>,
}

/// Hook 调用结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HookCallResult {
    /// 继续执行
    Continue,
    /// 修改数据后继续
    Modified(serde_json::Value),
    /// 阻止执行
    Blocked { reason: String },
    /// 跳过
    Skip,
    /// 替换为其他操作
    Replace(serde_json::Value),
}

/// WASM 插件导出函数定义
pub mod exports {
    // ========== 插件元数据 ==========

    /// 获取插件名称
    /// 返回: *const u8 (UTF-8 字符串)
    pub const FN_PLUGIN_NAME: &str = "wasm_plugin_name";

    /// 获取插件版本
    /// 返回: *const u8 (UTF-8 字符串)
    pub const FN_PLUGIN_VERSION: &str = "wasm_plugin_version";

    /// 获取插件类型
    /// 返回: i32 (0=Capability, 1=Hook, 2=Provider, 3=Channel)
    pub const FN_PLUGIN_TYPE: &str = "wasm_plugin_type";

    /// 获取插件支持的 Hooks
    /// 返回: *const u8 (JSON 数组)
    pub const FN_PLUGIN_HOOKS: &str = "wasm_plugin_hooks";

    /// 获取插件能力
    /// 返回: *const u8 (JSON 数组)
    pub const FN_PLUGIN_CAPABILITIES: &str = "wasm_plugin_capabilities";

    // ========== 生命周期 ==========

    /// 插件加载
    /// 参数: config: *const u8 (JSON)
    /// 返回: i32 (0=成功, 非0=错误码)
    pub const FN_ON_LOAD: &str = "wasm_plugin_on_load";

    /// 插件卸载
    /// 返回: i32 (0=成功, 非0=错误码)
    pub const FN_ON_UNLOAD: &str = "wasm_plugin_on_unload";

    /// 插件启用
    /// 返回: i32 (0=成功, 非0=错误码)
    pub const FN_ON_ENABLE: &str = "wasm_plugin_on_enable";

    /// 插件禁用
    /// 返回: i32 (0=成功, 非0=错误码)
    pub const FN_ON_DISABLE: &str = "wasm_plugin_on_disable";

    /// 配置更新
    /// 参数: config: *const u8 (JSON)
    /// 返回: i32 (0=成功, 非0=错误码)
    pub const FN_ON_CONFIG_UPDATE: &str = "wasm_plugin_on_config_update";

    // ========== Hook 处理 ==========

    /// Hook 处理函数
    /// 参数: hook_name: *const u8, context: *const u8 (JSON)
    /// 返回: *const u8 (JSON HookCallResult)
    pub const FN_ON_HOOK: &str = "wasm_plugin_on_hook";

    // ========== Agent 生命周期 ==========

    /// Agent 启动
    /// 参数: agent_id: *const u8
    pub const FN_ON_AGENT_START: &str = "wasm_plugin_on_agent_start";

    /// Agent 停止
    /// 参数: agent_id: *const u8
    pub const FN_ON_AGENT_STOP: &str = "wasm_plugin_on_agent_stop";

    /// 会话开始
    /// 参数: agent_id: *const u8, session_id: *const u8
    pub const FN_ON_SESSION_START: &str = "wasm_plugin_on_session_start";

    /// 会话结束
    /// 参数: agent_id: *const u8, session_id: *const u8
    pub const FN_ON_SESSION_END: &str = "wasm_plugin_on_session_end";

    // ========== 能力调用 ==========

    /// 调用能力
    /// 参数: name: *const u8, params: *const u8 (JSON)
    /// 返回: *const u8 (JSON)
    pub const FN_INVOKE_CAPABILITY: &str = "wasm_plugin_invoke_capability";
}

/// WASM 插件宿主函数定义
pub mod host_functions {
    // ========== 内存管理 ==========

    /// 分配内存
    /// 参数: size: usize
    /// 返回: *mut u8
    pub const FN_ALLOC: &str = "host_alloc";

    /// 释放内存
    /// 参数: ptr: *mut u8
    pub const FN_FREE: &str = "host_free";

    /// 读取字节
    /// 参数: ptr: *const u8, len: usize, out: *mut u8
    pub const FN_READ_BYTES: &str = "host_read_bytes";

    // ========== Agent 状态 ==========

    /// 获取 Agent 状态
    /// 返回: *const u8 (JSON)
    pub const FN_GET_AGENT_STATE: &str = "host_get_agent_state";

    // ========== 记忆系统 ==========

    /// 获取记忆
    /// 参数: key: *const u8
    /// 返回: *const u8 (JSON)
    pub const FN_GET_MEMORY: &str = "host_get_memory";

    /// 设置记忆
    /// 参数: key: *const u8, value: *const u8 (JSON)
    pub const FN_SET_MEMORY: &str = "host_set_memory";

    // ========== 插件间通信 ==========

    /// 调用其他插件
    /// 参数: plugin_id: *const u8, method: *const u8, params: *const u8 (JSON)
    /// 返回: *const u8 (JSON)
    pub const FN_CALL_PLUGIN: &str = "host_call_plugin";

    // ========== 日志 ==========

    /// 日志
    /// 参数: level: i32, message: *const u8
    pub const FN_LOG: &str = "host_log";

    // ========== 工具调用 ==========

    /// 调用工具
    /// 参数: tool_name: *const u8, params: *const u8 (JSON)
    /// 返回: *const u8 (JSON)
    pub const FN_CALL_TOOL: &str = "host_call_tool";

    // ========== 网络 ==========

    /// HTTP 请求
    /// 参数: method: *const u8, url: *const u8, body: *const u8
    /// 返回: *const u8 (JSON)
    pub const FN_HTTP_REQUEST: &str = "host_http_request";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_manifest_creation() {
        let manifest = PluginManifest::new("test-plugin", "1.0.0")
            .with_description("A test plugin")
            .with_author("Test Author")
            .with_plugin_type(PluginType::Hook)
            .with_capability("test_capability")
            .with_hook("on_input_received")
            .with_wasm_entry("test_plugin.wasm");

        assert_eq!(manifest.name, "test-plugin");
        assert_eq!(manifest.version, "1.0.0");
        assert_eq!(manifest.plugin_type, PluginType::Hook);
        assert_eq!(manifest.capabilities.len(), 1);
        assert_eq!(manifest.hooks.len(), 1);
    }

    #[test]
    fn test_plugin_manifest_validation() {
        let manifest =
            PluginManifest::new("test-plugin", "1.0.0").with_wasm_entry("test_plugin.wasm");

        assert!(manifest.validate().is_ok());

        let empty_manifest = PluginManifest::new("", "");
        assert!(empty_manifest.validate().is_err());
    }
}
