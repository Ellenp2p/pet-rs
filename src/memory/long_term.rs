//! 长期记忆模块
//!
//! 持久化存储记忆条目。

use super::memory_impl::MemoryEntry;
use crate::error::FrameworkError;
use std::collections::HashMap;

/// 长期记忆
pub struct LongTermMemory {
    /// 是否启用
    enabled: bool,
    /// 记忆条目
    entries: HashMap<String, MemoryEntry>,
}

impl LongTermMemory {
    /// 创建新的长期记忆
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            entries: HashMap::new(),
        }
    }

    /// 是否启用
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// 获取条目数
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// 存储条目
    pub fn store(&mut self, entry: MemoryEntry) -> Result<(), FrameworkError> {
        if !self.enabled {
            return Ok(());
        }

        self.entries.insert(entry.id.clone(), entry);
        Ok(())
    }

    /// 获取条目
    pub fn get(&self, id: &str) -> Option<&MemoryEntry> {
        if !self.enabled {
            return None;
        }

        self.entries.get(id)
    }

    /// 搜索条目
    pub fn search(&self, query: &str) -> Vec<&MemoryEntry> {
        if !self.enabled {
            return vec![];
        }

        self.entries
            .values()
            .filter(|e| {
                e.content.to_string().contains(query)
                    || e.tags.iter().any(|t: &String| t.contains(query))
            })
            .collect()
    }

    /// 删除条目
    pub fn remove(&mut self, id: &str) -> Option<MemoryEntry> {
        if !self.enabled {
            return None;
        }

        self.entries.remove(id)
    }

    /// 清空
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_long_term_memory() {
        let mut memory = LongTermMemory::new(true);

        let entry = MemoryEntry {
            id: "1".to_string(),
            content: serde_json::json!("test"),
            timestamp: 1234567890,
            tags: vec![],
            importance: 0.9,
        };

        memory.store(entry).unwrap();
        assert!(memory.get("1").is_some());
    }

    #[test]
    fn test_long_term_memory_disabled() {
        let mut memory = LongTermMemory::new(false);

        let entry = MemoryEntry {
            id: "1".to_string(),
            content: serde_json::json!("test"),
            timestamp: 1234567890,
            tags: vec![],
            importance: 0.9,
        };

        memory.store(entry).unwrap();
        assert!(memory.get("1").is_none());
    }
}
