//! 记忆系统示例
//!
//! 展示如何使用 agent-pet-rs 的记忆系统。
//! 演示短期记忆、长期记忆、工作记忆和压缩功能。
//!
//! ## 运行
//!
//! ```bash
//! cargo run --example memory_demo
//! ```

use agent_pet_rs::agent::core::MemoryConfig;
use agent_pet_rs::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧠 Memory System Demo");
    println!("=====================\n");

    // 创建记忆配置
    let config = MemoryConfig {
        short_term_capacity: 5,
        long_term_enabled: true,
        working_capacity: 3,
    };

    let mut memory = Memory::new(&config)?;

    println!("Memory configuration:");
    println!("- Short-term capacity: {}", config.short_term_capacity);
    println!("- Long-term enabled: {}", config.long_term_enabled);
    println!("- Working capacity: {}", config.working_capacity);

    println!("\n--- Adding memories ---\n");

    // 添加一些记忆
    let memories = vec![
        MemoryEntry {
            id: "1".to_string(),
            content: serde_json::json!({"type": "fact", "text": "The sky is blue"}),
            timestamp: 1000,
            tags: vec!["fact".to_string()],
            importance: 0.8,
        },
        MemoryEntry {
            id: "2".to_string(),
            content: serde_json::json!({"type": "event", "text": "User said hello"}),
            timestamp: 2000,
            tags: vec!["interaction".to_string()],
            importance: 0.5,
        },
        MemoryEntry {
            id: "3".to_string(),
            content: serde_json::json!({"type": "fact", "text": "Water is wet"}),
            timestamp: 3000,
            tags: vec!["fact".to_string()],
            importance: 0.9,
        },
        MemoryEntry {
            id: "4".to_string(),
            content: serde_json::json!({"type": "event", "text": "User asked about weather"}),
            timestamp: 4000,
            tags: vec!["interaction".to_string()],
            importance: 0.6,
        },
        MemoryEntry {
            id: "5".to_string(),
            content: serde_json::json!({"type": "fact", "text": "Rust is awesome"}),
            timestamp: 5000,
            tags: vec!["fact".to_string()],
            importance: 1.0,
        },
    ];

    for entry in memories {
        println!("Adding memory: {} - {:?}", entry.id, entry.content);
        memory.store(entry)?;
    }

    println!("\n--- Current state ---\n");
    println!("Short-term: {} entries", memory.short_term().len());
    println!("Long-term: {} entries", memory.long_term().len());
    println!("Working: {} entries", memory.working().len());

    println!("\n--- Searching memories ---\n");

    // 搜索记忆
    let results = memory.search("fact");
    println!("Search for 'fact': {} results", results.len());
    for result in &results {
        println!("- {}: {:?}", result.id, result.content);
    }

    println!("\n--- Retrieving specific memory ---\n");

    // 检索特定记忆
    if let Some(entry) = memory.retrieve("3") {
        println!("Retrieved memory 3: {:?}", entry.content);
    }

    println!("\n--- Memory compression ---\n");

    // 创建压缩器
    let compactor = MemoryCompactor::new(CompactionStrategy::ImportanceBased {
        min_importance: 0.7,
    });

    // 获取短期记忆中的所有条目
    let entries: Vec<MemoryEntry> = vec![]; // 这里需要从记忆中获取
    let compacted = compactor.compact(entries, 6000);

    println!("After compression (importance >= 0.7):");
    for entry in &compacted {
        println!("- {}: importance={}", entry.id, entry.importance);
    }

    println!("\n--- Memory persistence ---\n");

    // 测试持久化
    let temp_dir = std::env::temp_dir();
    let persistence_path = temp_dir.join("agent_memory_demo.json");
    let persistence = MemoryPersistence::new(&persistence_path);

    println!("Saving memories to: {:?}", persistence.storage_path());

    // 注意：这里只是演示 API，实际需要从记忆系统中获取条目
    let test_entries = vec![MemoryEntry {
        id: "test".to_string(),
        content: serde_json::json!({"text": "test memory"}),
        timestamp: 1234567890,
        tags: vec!["test".to_string()],
        importance: 0.5,
    }];

    persistence.save(&test_entries)?;
    println!("✅ Memories saved");

    let loaded = persistence.load()?;
    println!("✅ Memories loaded: {} entries", loaded.len());

    // 清理
    persistence.clear()?;
    println!("✅ Persistence cleared");

    Ok(())
}
