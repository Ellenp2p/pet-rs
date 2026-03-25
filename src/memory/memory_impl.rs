//! 记忆系统
//!
//! 管理短期记忆、长期记忆和工作记忆。

use serde::{Deserialize, Serialize};

use crate::agent::core::MemoryConfig;
use crate::error::FrameworkError;

use super::long_term::LongTermMemory;
use super::short_term::ShortTermMemory;
use super::working::WorkingMemory;

/// 记忆条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    /// 条目 ID
    pub id: String,
    /// 条目内容
    pub content: serde_json::Value,
    /// 时间戳
    pub timestamp: u64,
    /// 标签
    pub tags: Vec<String>,
    /// 重要性 (0.0 - 1.0)
    pub importance: f32,
}

/// 记忆系统
pub struct Memory {
    /// 短期记忆
    short_term: ShortTermMemory,
    /// 长期记忆
    long_term: LongTermMemory,
    /// 工作记忆
    working: WorkingMemory,
}

impl Memory {
    /// 创建新的记忆系统
    pub fn new(config: &MemoryConfig) -> Result<Self, FrameworkError> {
        Ok(Self {
            short_term: ShortTermMemory::new(config.short_term_capacity),
            long_term: LongTermMemory::new(config.long_term_enabled),
            working: WorkingMemory::new(config.working_capacity),
        })
    }

    /// 获取短期记忆
    pub fn short_term(&self) -> &ShortTermMemory {
        &self.short_term
    }

    /// 获取可变短期记忆
    pub fn short_term_mut(&mut self) -> &mut ShortTermMemory {
        &mut self.short_term
    }

    /// 获取长期记忆
    pub fn long_term(&self) -> &LongTermMemory {
        &self.long_term
    }

    /// 获取可变长期记忆
    pub fn long_term_mut(&mut self) -> &mut LongTermMemory {
        &mut self.long_term
    }

    /// 获取工作记忆
    pub fn working(&self) -> &WorkingMemory {
        &self.working
    }

    /// 获取可变工作记忆
    pub fn working_mut(&mut self) -> &mut WorkingMemory {
        &mut self.working
    }

    /// 存储记忆
    pub fn store(&mut self, entry: MemoryEntry) -> Result<(), FrameworkError> {
        // 存储到短期记忆
        self.short_term.store(entry.clone())?;

        // 如果重要性高，也存储到长期记忆
        if entry.importance > 0.7 {
            self.long_term.store(entry)?;
        }

        Ok(())
    }

    /// 检索记忆
    pub fn retrieve(&self, id: &str) -> Option<&MemoryEntry> {
        // 先检查工作记忆
        if let Some(entry) = self.working.get(id) {
            return Some(entry);
        }

        // 再检查短期记忆
        if let Some(entry) = self.short_term.get(id) {
            return Some(entry);
        }

        // 最后检查长期记忆
        self.long_term.get(id)
    }

    /// 搜索记忆
    pub fn search(&self, query: &str) -> Vec<&MemoryEntry> {
        let mut results = Vec::new();

        // 搜索工作记忆
        results.extend(self.working.search(query));

        // 搜索短期记忆
        results.extend(self.short_term.search(query));

        // 搜索长期记忆
        results.extend(self.long_term.search(query));

        results
    }

    /// 清空工作记忆
    pub fn clear_working(&mut self) {
        self.working.clear();
    }

    /// 压缩记忆
    pub fn compact(&mut self) -> Result<(), FrameworkError> {
        // 将工作记忆中的低重要性条目移动到短期记忆
        let entries = self.working.drain();
        for entry in entries {
            if entry.importance > 0.5 {
                self.short_term.store(entry)?;
            }
        }

        // 压缩短期记忆
        self.short_term.compact()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_creation() {
        let config = MemoryConfig {
            short_term_capacity: 100,
            long_term_enabled: true,
            working_capacity: 10,
        };
        let memory = Memory::new(&config).unwrap();
        assert_eq!(memory.short_term().capacity(), 100);
        assert_eq!(memory.working().capacity(), 10);
    }

    #[test]
    fn test_memory_store_retrieve() {
        let config = MemoryConfig {
            short_term_capacity: 100,
            long_term_enabled: true,
            working_capacity: 10,
        };
        let mut memory = Memory::new(&config).unwrap();

        let entry = MemoryEntry {
            id: "test-1".to_string(),
            content: serde_json::json!({"message": "hello"}),
            timestamp: 1234567890,
            tags: vec!["greeting".to_string()],
            importance: 0.8,
        };

        memory.store(entry).unwrap();
        assert!(memory.retrieve("test-1").is_some());
    }
}
