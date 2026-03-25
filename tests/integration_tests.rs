//! 集成测试
//!
//! 测试各个模块之间的协作。

use agent_pet_rs::agent::core::{MemoryConfig, PersonalityConfig, RoleConfig};
use agent_pet_rs::prelude::*;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

/// 测试 Agent 生命周期
#[test]
fn test_agent_lifecycle() {
    let config = AgentConfig::default();
    let mut agent = Agent::new(config).unwrap();

    // 启动 Agent
    assert!(agent.start().is_ok());
    assert!(matches!(agent.state(), AgentState::Idle));

    // 停止 Agent
    assert!(agent.stop().is_ok());
    assert!(matches!(agent.state(), AgentState::Idle));
}

/// 测试 Agent Hook 系统集成
#[test]
fn test_agent_hook_integration() {
    let config = AgentConfig::default();
    let mut agent = Agent::new(config).unwrap();

    let counter = Arc::new(AtomicU32::new(0));
    let c = counter.clone();

    // 注册 Hook
    agent
        .hooks_mut()
        .register(
            HookPoint::OnAgentStart,
            100,
            Arc::new(move |_ctx| {
                c.fetch_add(1, Ordering::SeqCst);
                Ok(HookResult::Continue)
            }),
        )
        .unwrap();

    // 启动 Agent 会触发 Hook
    agent.start().unwrap();
    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

/// 测试 Agent 人格切换
#[test]
fn test_agent_personality_switch() {
    let config = AgentConfig::default();
    let mut agent = Agent::new(config).unwrap();

    let initial_personality = agent.personality().name().to_string();

    let new_config = PersonalityConfig {
        name: "Serious".to_string(),
        description: "A serious agent".to_string(),
        traits: vec!["serious".to_string()],
        dialogue_style: "formal".to_string(),
    };
    let new_personality = Personality::from_config(&new_config).unwrap();

    assert!(agent.switch_personality(new_personality).is_ok());
    assert_ne!(agent.personality().name(), initial_personality);
}

/// 测试 Agent 角色切换
#[test]
fn test_agent_role_switch() {
    let config = AgentConfig::default();
    let mut agent = Agent::new(config).unwrap();

    let new_config = RoleConfig {
        name: "Admin".to_string(),
        description: "Administrator".to_string(),
        capabilities: vec!["admin".to_string(), "manage".to_string()],
    };
    let new_role = Role::from_config(&new_config).unwrap();

    assert!(agent.switch_role(new_role).is_ok());
    assert!(agent.role().has_capability("admin"));
}

/// 测试 Hook 系统与插件协作
#[test]
fn test_hook_plugin_collaboration() {
    let mut registry = HookRegistry::default();

    let input_counter = Arc::new(AtomicU32::new(0));
    let action_counter = Arc::new(AtomicU32::new(0));

    let c1 = input_counter.clone();
    registry
        .register(
            HookPoint::OnInputReceived,
            100,
            Arc::new(move |_ctx| {
                c1.fetch_add(1, Ordering::SeqCst);
                Ok(HookResult::Continue)
            }),
        )
        .unwrap();

    let c2 = action_counter.clone();
    registry
        .register(
            HookPoint::BeforeAction,
            100,
            Arc::new(move |_ctx| {
                c2.fetch_add(1, Ordering::SeqCst);
                Ok(HookResult::Continue)
            }),
        )
        .unwrap();

    // 触发 Hook
    let ctx = HookContext::new(HookPoint::OnInputReceived, "agent-1".to_string());
    registry.trigger("on_input_received", &ctx).unwrap();

    let ctx = HookContext::new(HookPoint::BeforeAction, "agent-1".to_string());
    registry.trigger("before_action", &ctx).unwrap();

    assert_eq!(input_counter.load(Ordering::SeqCst), 1);
    assert_eq!(action_counter.load(Ordering::SeqCst), 1);
}

/// 测试 Hook 阻止机制
#[test]
fn test_hook_blocking() {
    let mut registry = HookRegistry::default();

    registry
        .register(
            HookPoint::BeforeAction,
            100,
            Arc::new(|_ctx| {
                Ok(HookResult::Blocked {
                    reason: "Action blocked".to_string(),
                })
            }),
        )
        .unwrap();

    let ctx = HookContext::new(HookPoint::BeforeAction, "agent-1".to_string());
    let result = registry.trigger("before_action", &ctx).unwrap();

    assert!(result.is_blocked());
}

/// 测试决策引擎与 Hook 集成
#[test]
fn test_decision_hook_integration() {
    let mut engine = RuleBasedEngine::new();

    engine.add_rule(Rule {
        name: "greeting".to_string(),
        condition: Box::new(|ctx| ctx.input.contains("hello")),
        action: Box::new(|_| Decision {
            decision_type: DecisionType::Reply,
            content: serde_json::json!({"message": "Hello!"}),
            confidence: 0.9,
            reason: Some("Greeting".to_string()),
        }),
        priority: 100,
    });

    let context = DecisionContext {
        input: "hello there".to_string(),
        history: vec![],
        available_tools: vec![],
        agent_state: serde_json::json!({}),
    };

    let decision = engine.decide(&context).unwrap();
    assert!(matches!(decision.decision_type, DecisionType::Reply));
    assert!(decision.confidence > 0.5);
}

/// 测试记忆系统与压缩
#[test]
fn test_memory_with_compaction() {
    use agent_pet_rs::agent::core::MemoryConfig;

    let config = MemoryConfig {
        short_term_capacity: 10,
        long_term_enabled: true,
        working_capacity: 5,
    };

    let mut memory = Memory::new(&config).unwrap();

    // 添加一些记忆
    for i in 0..5 {
        memory
            .store(MemoryEntry {
                id: format!("{}", i),
                content: serde_json::json!({"text": format!("memory {}", i)}),
                timestamp: i as u64 * 1000,
                tags: vec!["test".to_string()],
                importance: 0.5 + (i as f32 * 0.1),
            })
            .unwrap();
    }

    // 验证记忆存储
    assert!(memory.retrieve("0").is_some());
    assert!(memory.retrieve("4").is_some());

    // 搜索记忆 (可能在多个记忆系统中找到)
    let results = memory.search("memory");
    assert!(results.len() >= 5); // 至少有 5 条
}

/// 测试插件 Slot 系统
#[test]
fn test_slot_system() {
    let mut manager = SlotManager::new();

    // 注册 Slot
    assert!(manager
        .register(Slot::DecisionEngine, "plugin-1".to_string())
        .is_ok());

    // 独占 Slot 不能重复注册
    assert!(manager
        .register(Slot::DecisionEngine, "plugin-2".to_string())
        .is_err());

    // 注销 Slot
    assert!(manager
        .unregister(&Slot::DecisionEngine, "plugin-1")
        .is_ok());
}

/// 测试 Capability 系统
#[test]
fn test_capability_system() {
    let mut registry = CapabilityRegistry::new();

    // 注册能力
    assert!(registry
        .register(Capability::Tool, "plugin-1".to_string(), 100)
        .is_ok());
    assert!(registry
        .register(Capability::Tool, "plugin-2".to_string(), 50)
        .is_ok());

    // 获取最佳提供者
    let best = registry.get_best_provider(&Capability::Tool);
    assert!(best.is_some());
    assert_eq!(best.unwrap().plugin_id, "plugin-2"); // 更高优先级
}

/// 测试上下文管理
#[test]
fn test_context_management() {
    let mut context = Context::new();
    context.session_id = Some("session-1".to_string());
    context.agent_id = Some("agent-1".to_string());
    context.add_history("user".to_string(), "hello".to_string());
    context.add_history("assistant".to_string(), "hi".to_string());

    assert_eq!(context.history_len(), 2);
    assert_eq!(context.session_id, Some("session-1".to_string()));
    assert_eq!(context.agent_id, Some("agent-1".to_string()));
}

/// 测试通信系统
#[test]
fn test_communication_system() {
    let message = Message::text("user".to_string(), "hello".to_string());
    assert_eq!(message.sender, "user");
    assert_eq!(message.content, "hello");
    assert!(matches!(message.message_type, MessageType::Text));
}

/// 测试错误处理
#[test]
fn test_error_handling() {
    let error = FrameworkError::Other("test error".to_string());
    assert!(error.to_string().contains("test error"));
}

/// 测试完整 Agent 工作流
#[test]
fn test_full_agent_workflow() {
    // 创建 Agent
    let config = AgentConfig::default();
    let mut agent = Agent::new(config).unwrap();

    // 注册 Hook
    let hook_called = Arc::new(AtomicU32::new(0));
    let c = hook_called.clone();
    agent
        .hooks_mut()
        .register(
            HookPoint::OnAgentStart,
            100,
            Arc::new(move |_ctx| {
                c.fetch_add(1, Ordering::SeqCst);
                Ok(HookResult::Continue)
            }),
        )
        .unwrap();

    // 启动 Agent
    agent.start().unwrap();
    assert_eq!(hook_called.load(Ordering::SeqCst), 1);

    // 创建决策引擎
    let mut engine = RuleBasedEngine::new();
    engine.add_rule(Rule {
        name: "default".to_string(),
        condition: Box::new(|_| true),
        action: Box::new(|_| Decision {
            decision_type: DecisionType::Reply,
            content: serde_json::json!({"message": "Default response"}),
            confidence: 0.5,
            reason: None,
        }),
        priority: 0,
    });

    // 创建记忆
    let memory_config = MemoryConfig {
        short_term_capacity: 100,
        long_term_enabled: true,
        working_capacity: 10,
    };
    let mut memory = Memory::new(&memory_config).unwrap();
    memory
        .store(MemoryEntry {
            id: "test".to_string(),
            content: serde_json::json!({"text": "test memory"}),
            timestamp: 1000,
            tags: vec!["test".to_string()],
            importance: 0.8,
        })
        .unwrap();

    // 验证工作流
    assert!(memory.retrieve("test").is_some());

    let context = DecisionContext {
        input: "test".to_string(),
        history: vec![],
        available_tools: vec![],
        agent_state: serde_json::json!({}),
    };
    let decision = engine.decide(&context).unwrap();
    assert!(matches!(decision.decision_type, DecisionType::Reply));

    // 停止 Agent
    agent.stop().unwrap();
    assert!(matches!(agent.state(), AgentState::Idle));
}
