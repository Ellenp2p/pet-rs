//! Hook 系统示例
//!
//! 展示如何使用 agent-pet-rs 的 Hook 系统。
//! 演示 28 个 Hook 点的注册和触发。
//!
//! ## 运行
//!
//! ```bash
//! cargo run --example hook_demo
//! ```

use agent_pet_rs::prelude::*;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔗 Hook System Demo");
    println!("===================\n");

    let mut registry = HookRegistry::default();

    // 注册多个 Hook
    let input_counter = Arc::new(AtomicU32::new(0));
    let action_counter = Arc::new(AtomicU32::new(0));
    let decision_counter = Arc::new(AtomicU32::new(0));

    let c1 = input_counter.clone();
    registry.register(
        HookPoint::OnInputReceived,
        100,
        Arc::new(move |_ctx| {
            c1.fetch_add(1, Ordering::SeqCst);
            println!("  📥 Input received");
            Ok(HookResult::Continue)
        }),
    )?;

    let c2 = action_counter.clone();
    registry.register(
        HookPoint::BeforeAction,
        100,
        Arc::new(move |_ctx| {
            c2.fetch_add(1, Ordering::SeqCst);
            println!("  ⚡ Before action");
            Ok(HookResult::Continue)
        }),
    )?;

    let c3 = decision_counter.clone();
    registry.register(
        HookPoint::BeforeDecision,
        100,
        Arc::new(move |_ctx| {
            c3.fetch_add(1, Ordering::SeqCst);
            println!("  🤔 Before decision");
            Ok(HookResult::Continue)
        }),
    )?;

    // 注册一个可以修改数据的 Hook
    registry.register(
        HookPoint::BeforeAction,
        50, // 高优先级
        Arc::new(|ctx| {
            println!("  🔍 Validating action...");
            if let Some(data) = ctx.get_data("action") {
                if data == "dangerous" {
                    println!("  ⚠️ Dangerous action detected!");
                    return Ok(HookResult::Blocked {
                        reason: "Dangerous action blocked".to_string(),
                    });
                }
            }
            Ok(HookResult::Continue)
        }),
    )?;

    println!("Registered hooks:");
    println!(
        "- on_input_received: {} callbacks",
        registry.count(HookPoint::OnInputReceived)
    );
    println!(
        "- before_action: {} callbacks",
        registry.count(HookPoint::BeforeAction)
    );
    println!(
        "- before_decision: {} callbacks",
        registry.count(HookPoint::BeforeDecision)
    );

    println!("\n--- Simulating events ---\n");

    // 模拟触发 Hook
    println!("Event 1: Normal input");
    let ctx = HookContext::new(HookPoint::OnInputReceived, "agent-1".to_string());
    registry.trigger("on_input_received", &ctx)?;

    println!("\nEvent 2: Before decision");
    let ctx = HookContext::new(HookPoint::BeforeDecision, "agent-1".to_string());
    registry.trigger("before_decision", &ctx)?;

    println!("\nEvent 3: Before action (normal)");
    let ctx = HookContext::new(HookPoint::BeforeAction, "agent-1".to_string());
    registry.trigger("before_action", &ctx)?;

    println!("\nEvent 4: Before action (dangerous)");
    let mut ctx = HookContext::new(HookPoint::BeforeAction, "agent-1".to_string());
    ctx.set_data("action".to_string(), serde_json::json!("dangerous"));
    match registry.trigger("before_action", &ctx)? {
        HookResult::Blocked { reason } => {
            println!("  ❌ Action blocked: {}", reason);
        }
        _ => {
            println!("  ✅ Action allowed");
        }
    }

    println!("\n📊 Statistics:");
    println!("- Input received: {}", input_counter.load(Ordering::SeqCst));
    println!("- Before action: {}", action_counter.load(Ordering::SeqCst));
    println!(
        "- Before decision: {}",
        decision_counter.load(Ordering::SeqCst)
    );

    Ok(())
}
