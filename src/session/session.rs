//! 会话类型

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::channels::SessionId;

/// 会话类型
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SessionType {
    /// 主会话 (完全信任，完整权限)
    Main,
    /// 私聊会话 (部分权限)
    Direct,
    /// 群组会话 (沙箱隔离)
    Group,
}

impl std::fmt::Display for SessionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionType::Main => write!(f, "main"),
            SessionType::Direct => write!(f, "direct"),
            SessionType::Group => write!(f, "group"),
        }
    }
}

/// 会话权限
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionPermissions {
    /// 是否允许执行命令
    pub allow_commands: bool,
    /// 是否允许访问文件系统
    pub allow_filesystem: bool,
    /// 是否允许网络访问
    pub allow_network: bool,
    /// 是否允许访问 secrets
    pub allow_secrets: bool,
    /// 允许的命令列表 (空 = 全部允许)
    pub allowed_commands: Vec<String>,
    /// 允许的网络域名 (空 = 全部允许)
    pub allowed_domains: Vec<String>,
}

impl SessionPermissions {
    /// 主会话权限 (完全信任)
    pub fn main() -> Self {
        Self {
            allow_commands: true,
            allow_filesystem: true,
            allow_network: true,
            allow_secrets: true,
            allowed_commands: vec![],
            allowed_domains: vec![],
        }
    }

    /// 私聊权限 (部分信任)
    pub fn direct() -> Self {
        Self {
            allow_commands: true,
            allow_filesystem: false,
            allow_network: true,
            allow_secrets: false,
            allowed_commands: vec![],
            allowed_domains: vec![],
        }
    }

    /// 群组权限 (沙箱)
    pub fn group() -> Self {
        Self {
            allow_commands: false,
            allow_filesystem: false,
            allow_network: false,
            allow_secrets: false,
            allowed_commands: vec![],
            allowed_domains: vec![],
        }
    }
}

impl Default for SessionPermissions {
    fn default() -> Self {
        Self::group()
    }
}

/// 会话配置
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionConfig {
    /// 会话类型
    pub session_type: SessionType,
    /// 权限
    pub permissions: SessionPermissions,
    /// 最大空闲时间 (秒)
    pub max_idle_secs: u64,
    /// 是否持久化
    pub persistent: bool,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            session_type: SessionType::Direct,
            permissions: SessionPermissions::default(),
            max_idle_secs: 3600, // 1 小时
            persistent: true,
        }
    }
}

/// 会话记忆 (会话级)
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SessionMemory {
    /// 对话历史
    pub history: Vec<HistoryEntry>,
    /// 长期记忆
    pub long_term: HashMap<String, String>,
    /// 工作记忆
    pub working: HashMap<String, String>,
}

/// 历史记录条目
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub role: String,
    pub content: String,
    pub timestamp: u64,
}

/// 会话
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Session {
    /// 会话 ID
    pub id: SessionId,
    /// 会话类型
    pub session_type: SessionType,
    /// 创建时间
    pub created_at: u64,
    /// 最后活动时间
    pub last_active: u64,
    /// 会话记忆
    pub memory: SessionMemory,
    /// 权限
    pub permissions: SessionPermissions,
    /// 元数据
    pub metadata: HashMap<String, String>,
}

impl Session {
    /// 创建新会话
    pub fn new(id: SessionId, session_type: SessionType) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let permissions = match session_type {
            SessionType::Main => SessionPermissions::main(),
            SessionType::Direct => SessionPermissions::direct(),
            SessionType::Group => SessionPermissions::group(),
        };

        Self {
            id,
            session_type,
            created_at: now,
            last_active: now,
            memory: SessionMemory::default(),
            permissions,
            metadata: HashMap::new(),
        }
    }

    /// 更新最后活动时间
    pub fn touch(&mut self) {
        self.last_active = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }

    /// 添加历史记录
    pub fn add_history(&mut self, role: &str, content: &str) {
        self.memory.history.push(HistoryEntry {
            role: role.to_string(),
            content: content.to_string(),
            timestamp: self.last_active,
        });
    }

    /// 检查是否过期
    pub fn is_expired(&self, max_idle_secs: u64) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now - self.last_active > max_idle_secs
    }
}
