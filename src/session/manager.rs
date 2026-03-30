//! 会话管理器

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::session::{Session, SessionConfig, SessionType};
use crate::channels::SessionId;

/// 会话管理器
pub struct SessionManager {
    /// 会话存储
    sessions: Arc<RwLock<HashMap<SessionId, Session>>>,
    /// 默认配置
    default_config: SessionConfig,
}

impl SessionManager {
    /// 创建新的会话管理器
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            default_config: SessionConfig::default(),
        }
    }

    /// 使用自定义配置创建
    pub fn with_config(config: SessionConfig) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            default_config: config,
        }
    }

    /// 获取或创建会话
    pub async fn get_or_create(&self, id: &SessionId, session_type: SessionType) -> Session {
        let mut sessions = self.sessions.write().await;
        sessions
            .entry(id.clone())
            .or_insert_with(|| Session::new(id.clone(), session_type))
            .clone()
    }

    /// 获取会话
    pub async fn get(&self, id: &SessionId) -> Option<Session> {
        let sessions = self.sessions.read().await;
        sessions.get(id).cloned()
    }

    /// 更新会话
    pub async fn update(&self, session: Session) {
        let mut sessions = self.sessions.write().await;
        sessions.insert(session.id.clone(), session);
    }

    /// 删除会话
    pub async fn remove(&self, id: &SessionId) -> Option<Session> {
        let mut sessions = self.sessions.write().await;
        sessions.remove(id)
    }

    /// 清理过期会话
    pub async fn cleanup_expired(&self) -> usize {
        let mut sessions = self.sessions.write().await;
        let max_idle = self.default_config.max_idle_secs;
        let before = sessions.len();
        sessions.retain(|_, session| !session.is_expired(max_idle));
        before - sessions.len()
    }

    /// 获取会话数量
    pub async fn count(&self) -> usize {
        let sessions = self.sessions.read().await;
        sessions.len()
    }

    /// 列出所有会话 ID
    pub async fn list_ids(&self) -> Vec<SessionId> {
        let sessions = self.sessions.read().await;
        sessions.keys().cloned().collect()
    }

    /// 根据渠道获取会话
    pub async fn list_by_channel(&self, channel: &str) -> Vec<Session> {
        let sessions = self.sessions.read().await;
        sessions
            .values()
            .filter(|s| s.id.channel == channel)
            .cloned()
            .collect()
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}
