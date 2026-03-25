//! 工作记忆模块
//!
//! 存储当前会话正在使用的记忆条目。

use super::memory_impl::MemoryEntry;
use crate::error::FrameworkError;
use std::collections::HashMap;

/// 工作记忆
pub struct WorkingMemory {
    /// 容量
    capacity: usize,
    /// 记忆条目
    entries: HashMap<String, MemoryEntry>,
}

impl WorkingMemory {
    /// 创建新的工作记忆
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            entries: HashMap::with_capacity(capacity),
        }
    }

    /// 获取容量
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// 获取当前条目数
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// 存储条目
    pub fn store(&mut self, entry: MemoryEntry) -> Result<(), FrameworkError> {
        // 如果已满，移除最旧的条目
        if self.entries.len() >= self.capacity {
            if let Some(oldest_id) = self
                .entries
                .values()
                .min_by_key(|e| e.timestamp)
                .map(|e| e.id.clone())
            {
                self.entries.remove(&oldest_id);
            }
        }

        self.entries.insert(entry.id.clone(), entry);
        Ok(())
    }

    /// 获取条目
    pub fn get(&self, id: &str) -> Option<&MemoryEntry> {
        self.entries.get(id)
    }

    /// 搜索条目
    pub fn search(&self, query: &str) -> Vec<&MemoryEntry> {
        self.entries
            .values()
            .filter(|e| {
                e.content.to_string().contains(query)
                    || e.tags.iter().any(|t: &String| t.contains(query))
            })
            .collect()
    }

    /// 移除条目
    pub fn remove(&mut self, id: &str) -> Option<MemoryEntry> {
        self.entries.remove(id)
    }

    /// 清空
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// 排空所有条目
    pub fn drain(&mut self) -> Vec<MemoryEntry> {
        self.entries.drain().map(|(_, e)| e).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_working_memory() {
        let mut memory = WorkingMemory::new(3);

        let entry = MemoryEntry {
            id: "1".to_string(),
            content: serde_json::json!("test"),
            timestamp: 1234567890,
            tags: vec![],
            importance: 0.5,
        };

        memory.store(entry).unwrap();
        assert_eq!(memory.len(), 1);
        assert!(memory.get("1").is_some());
    }
}
