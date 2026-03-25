//! 决策引擎示例
//!
//! 展示如何使用 agent-pet-rs 的决策引擎。
//! 演示规则引擎、LLM 引擎和混合引擎。
//!
//! ## 运行
//!
//! ```bash
//! cargo run --example decision_demo
//! ```

use agent_pet_rs::decision::{
    Decision, DecisionContext, DecisionEngineTrait, DecisionType, HybridConfig, HybridEngine,
    HybridStrategy, LLMConfig, LLMEngine, Rule, RuleBasedEngine,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Decision Engine Demo");
    println!("=======================\n");

    // 创建规则引擎
    let mut rule_engine = RuleBasedEngine::new();

    // 添加问候规则
    rule_engine.add_rule(Rule {
        name: "greeting".to_string(),
        condition: Box::new(|ctx| {
            ctx.input.to_lowercase().contains("hello") || ctx.input.to_lowercase().contains("hi")
        }),
        action: Box::new(|_| Decision {
            decision_type: DecisionType::Reply,
            content: serde_json::json!({"message": "Hello! Nice to meet you!"}),
            confidence: 0.95,
            reason: Some("Greeting detected".to_string()),
        }),
        priority: 100,
    });

    // 添加帮助规则
    rule_engine.add_rule(Rule {
        name: "help".to_string(),
        condition: Box::new(|ctx| ctx.input.to_lowercase().contains("help")),
        action: Box::new(|_| Decision {
            decision_type: DecisionType::Reply,
            content: serde_json::json!({"message": "I'm here to help! What do you need?"}),
            confidence: 0.9,
            reason: Some("Help request detected".to_string()),
        }),
        priority: 90,
    });

    // 添加工具调用规则
    rule_engine.add_rule(Rule {
        name: "weather".to_string(),
        condition: Box::new(|ctx| ctx.input.to_lowercase().contains("weather")),
        action: Box::new(|_| Decision {
            decision_type: DecisionType::ToolCall,
            content: serde_json::json!({
                "tool": "weather",
                "params": {"location": "current"}
            }),
            confidence: 0.85,
            reason: Some("Weather query detected".to_string()),
        }),
        priority: 80,
    });

    println!("Registered rules:");
    println!("- greeting (priority: 100)");
    println!("- help (priority: 90)");
    println!("- weather (priority: 80)");

    // 测试不同的输入
    let test_inputs = vec![
        "Hello there!",
        "Can you help me?",
        "What's the weather like?",
        "Tell me a joke",
        "Hi!",
    ];

    println!("\n--- Testing rule engine ---\n");

    for input in test_inputs {
        let context = DecisionContext {
            input: input.to_string(),
            history: vec![],
            available_tools: vec!["weather".to_string(), "calculator".to_string()],
            agent_state: serde_json::json!({}),
        };

        println!("Input: \"{}\"", input);

        match rule_engine.decide(&context) {
            Ok(decision) => {
                println!(
                    "  Decision: {:?} (confidence: {:.2})",
                    decision.decision_type, decision.confidence
                );
                println!("  Content: {}", decision.content);
                if let Some(reason) = &decision.reason {
                    println!("  Reason: {}", reason);
                }
            }
            Err(e) => {
                println!("  Error: {}", e);
            }
        }
        println!();
    }

    println!("--- Testing hybrid engine ---\n");

    // 创建混合引擎
    let llm_config = LLMConfig {
        provider: "openai".to_string(),
        model: "gpt-3.5-turbo".to_string(),
        ..Default::default()
    };
    let llm_engine = LLMEngine::new(llm_config);

    let hybrid_config = HybridConfig {
        strategy: HybridStrategy::RuleFirst,
        llm_enabled: true,
        llm_confidence_threshold: 0.7,
        ..Default::default()
    };

    let hybrid_engine = HybridEngine::new(rule_engine, llm_engine, hybrid_config);

    let context = DecisionContext {
        input: "Hello!".to_string(),
        history: vec![],
        available_tools: vec![],
        agent_state: serde_json::json!({}),
    };

    println!("Input: \"Hello!\"");
    match hybrid_engine.decide(&context) {
        Ok(decision) => {
            println!(
                "  Decision: {:?} (confidence: {:.2})",
                decision.decision_type, decision.confidence
            );
            println!("  Content: {}", decision.content);
        }
        Err(e) => {
            println!("  Error: {}", e);
        }
    }

    Ok(())
}
