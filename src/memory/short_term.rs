//! 短期记忆模块
//!
//! 存储最近的记忆条目。

use super::memory_impl::MemoryEntry;
use crate::error::FrameworkError;
use std::collections::VecDeque;

/// 短期记忆
pub struct ShortTermMemory {
    /// 容量
    capacity: usize,
    /// 记忆条目
    entries: VecDeque<MemoryEntry>,
}

impl ShortTermMemory {
    /// 创建新的短期记忆
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            entries: VecDeque::with_capacity(capacity),
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
            self.entries.pop_front();
        }

        self.entries.push_back(entry);
        Ok(())
    }

    /// 获取条目
    pub fn get(&self, id: &str) -> Option<&MemoryEntry> {
        self.entries.iter().find(|e| e.id == id)
    }

    /// 搜索条目
    pub fn search(&self, query: &str) -> Vec<&MemoryEntry> {
        self.entries
            .iter()
            .filter(|e| {
                e.content.to_string().contains(query)
                    || e.tags.iter().any(|t: &String| t.contains(query))
            })
            .collect()
    }

    /// 清空
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// 压缩
    pub fn compact(&mut self) -> Result<(), FrameworkError> {
        // 按重要性排序，保留重要的条目
        let mut entries: Vec<MemoryEntry> = self.entries.drain(..).collect();
        entries.sort_by(|a, b| b.importance.partial_cmp(&a.importance).unwrap());

        // 只保留一半的容量
        let keep = self.capacity / 2;
        entries.truncate(keep);

        self.entries = entries.into();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_short_term_memory() {
        let mut memory = ShortTermMemory::new(3);

        let entry1 = MemoryEntry {
            id: "1".to_string(),
            content: serde_json::json!("test1"),
            timestamp: 1,
            tags: vec![],
            importance: 0.5,
        };

        let entry2 = MemoryEntry {
            id: "2".to_string(),
            content: serde_json::json!("test2"),
            timestamp: 2,
            tags: vec![],
            importance: 0.6,
        };

        memory.store(entry1).unwrap();
        memory.store(entry2).unwrap();

        assert_eq!(memory.len(), 2);
        assert!(memory.get("1").is_some());
        assert!(memory.get("2").is_some());
    }
}
