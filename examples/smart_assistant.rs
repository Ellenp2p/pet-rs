//! 智能助手示例
//!
//! 展示如何使用 agent-pet-rs 框架创建一个智能助手。
//! 支持记忆、决策、Hook 系统等功能。
//!
//! ## 运行
//!
//! ```bash
//! cargo run --example smart_assistant
//! ```

use agent_pet_rs::agent::core::{DecisionConfig, MemoryConfig, PersonalityConfig, RoleConfig};
use agent_pet_rs::prelude::*;
use std::io::{self, BufRead, Write};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

/// 智能助手
struct SmartAssistant {
    agent: Agent,
    interaction_count: Arc<AtomicU32>,
}

impl SmartAssistant {
    fn new() -> Result<Self, FrameworkError> {
        // 创建 Agent 配置
        let config = AgentConfig {
            name: "Alice".to_string(),
            description: "A helpful smart assistant".to_string(),
            personality: PersonalityConfig {
                name: "Friendly".to_string(),
                description: "Friendly and helpful".to_string(),
                traits: vec!["friendly".to_string(), "helpful".to_string()],
                dialogue_style: "casual".to_string(),
            },
            role: RoleConfig {
                name: "Assistant".to_string(),
                description: "A helpful assistant".to_string(),
                capabilities: vec!["chat".to_string(), "help".to_string()],
            },
            memory: MemoryConfig {
                short_term_capacity: 50,
                long_term_enabled: true,
                working_capacity: 10,
            },
            decision: DecisionConfig {
                engine_type: agent_pet_rs::agent::core::DecisionEngineType::RuleBased,
                llm_provider: None,
                llm_model: None,
            },
        };

        let mut agent = Agent::new(config)?;

        // 注册 Hook
        let counter = Arc::new(AtomicU32::new(0));
        let c = counter.clone();

        agent.hooks_mut().register(
            HookPoint::OnInputReceived,
            100,
            Arc::new(move |_ctx| {
                c.fetch_add(1, Ordering::SeqCst);
                Ok(HookResult::Continue)
            }),
        )?;

        // 启动 Agent
        agent.start()?;

        Ok(Self {
            agent,
            interaction_count: counter,
        })
    }

    fn process_input(&mut self, input: &str) -> Result<String, FrameworkError> {
        // 触发 Hook
        let ctx = HookContext::new(HookPoint::OnInputReceived, self.agent.id().to_string());
        self.agent.hooks().trigger("on_input_received", &ctx)?;

        // 简单的规则匹配
        let response = if input.contains("hello") || input.contains("hi") {
            "Hello! How can I help you today?".to_string()
        } else if input.contains("help") {
            "I'm here to help! You can ask me questions or just chat.".to_string()
        } else if input.contains("bye") || input.contains("goodbye") {
            "Goodbye! Have a great day!".to_string()
        } else if input.contains("name") {
            format!("My name is {}!", self.agent.name())
        } else if input.contains("mood") || input.contains("how are you") {
            "I'm doing great, thank you for asking!".to_string()
        } else {
            format!(
                "I heard you say: '{}'. That's interesting! Tell me more.",
                input
            )
        };

        Ok(response)
    }

    fn get_stats(&self) -> String {
        format!(
            "Interactions: {}",
            self.interaction_count.load(Ordering::SeqCst)
        )
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 Smart Assistant Demo");
    println!("========================\n");

    let mut assistant = SmartAssistant::new()?;

    println!(
        "Assistant: Hello! I'm {}. How can I help you?",
        assistant.agent.name()
    );
    println!("(Type 'quit' to exit, 'stats' to see statistics)\n");

    loop {
        print!("You: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().lock().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        if input == "quit" || input == "exit" {
            println!("\nAssistant: Goodbye! Have a great day!");
            break;
        }

        if input == "stats" {
            println!("\n📊 {}", assistant.get_stats());
            continue;
        }

        match assistant.process_input(input) {
            Ok(response) => println!("Assistant: {}\n", response),
            Err(e) => println!("Error: {}\n", e),
        }
    }

    Ok(())
}
