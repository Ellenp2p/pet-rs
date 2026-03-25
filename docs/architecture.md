# agent-pet-rs 架构文档

## 概述

agent-pet-rs 是一个基于 WASM 插件的智能 Agent 框架。框架采用分层架构，核心与渲染层分离，支持多种渲染系统（Bevy GUI、CLI/TUI）。

## 核心设计原则

1. **领域无关** - 核心框架（src/）不包含任何领域特定代码
2. **事件驱动** - 所有状态变更通过事件系统
3. **插件扩展** - 通过 WASM 插件扩展功能
4. **类型安全** - 使用 Rust 类型系统确保安全

## 架构层次

```
┌─────────────────────────────────────────────────────────────┐
│                    示例层 (examples/)                        │
│  Bevy GUI / CLI/TUI / 其他渲染系统                          │
├─────────────────────────────────────────────────────────────┤
│                    适配器层 (examples/)                      │
│  BevyAdapter / CLIAdapter / 其他适配器                      │
├─────────────────────────────────────────────────────────────┤
│                    核心层 (src/)                             │
│  Agent / Hook / Plugin / Decision / Memory / Context        │
└─────────────────────────────────────────────────────────────┘
```

## 核心模块

### 1. Agent 核心 (`src/agent/`)

- **core.rs** - Agent 核心结构和状态管理
- **loop_impl.rs** - Agent 主循环
- **personality.rs** - 人格系统
- **role.rs** - 角色系统

### 2. Hook 系统 (`src/hooks/`)

- **points.rs** - 28 个 Hook 点定义
- **registry.rs** - Hook 注册表
- **runner.rs** - Hook 执行器
- **context.rs** - Hook 上下文

### 3. 插件系统 (`src/plugins/`)

- **manifest.rs** - 插件 Manifest
- **loader.rs** - 插件加载器
- **discovery.rs** - 插件发现
- **validator.rs** - 插件验证器
- **lifecycle.rs** - 生命周期管理
- **slots.rs** - Slot 系统
- **capabilities.rs** - Capability 模型

### 4. 决策引擎 (`src/decision/`)

- **engine.rs** - 决策引擎 trait
- **rule_based.rs** - 规则引擎
- **llm_based.rs** - LLM 引擎
- **hybrid.rs** - 混合引擎

### 5. 记忆系统 (`src/memory/`)

- **memory_impl.rs** - 记忆管理器
- **short_term.rs** - 短期记忆
- **long_term.rs** - 长期记忆
- **working.rs** - 工作记忆
- **compaction.rs** - 压缩和持久化

### 6. 上下文管理 (`src/context/`)

- **context_impl.rs** - 上下文结构
- **builder.rs** - 上下文构建器
- **window.rs** - 上下文窗口

### 7. 通信层 (`src/communication/`)

- **channel.rs** - 通道接口
- **message.rs** - 消息定义
- **router.rs** - 消息路由器

## Hook 系统详解

### Hook 点分类

1. **输入处理层** (3 个)
   - on_input_received
   - before_input_parse
   - after_input_parse

2. **上下文构建层** (4 个)
   - before_context_build
   - after_context_build
   - before_memory_load
   - after_memory_load

3. **决策层** (4 个)
   - before_decision
   - after_decision
   - before_llm_call
   - after_llm_call

4. **动作执行层** (4 个)
   - before_action
   - after_action
   - before_tool_call
   - after_tool_call

5. **输出生成层** (3 个)
   - before_output
   - after_output
   - before_response

6. **记忆管理层** (3 个)
   - before_memory_write
   - after_memory_write
   - before_memory_compact

7. **角色/人格层** (3 个)
   - before_role_apply
   - after_role_apply
   - on_personality_change

8. **生命周期层** (4 个)
   - on_agent_start
   - on_agent_stop
   - on_session_start
   - on_session_end

### Hook 执行模式

- **Sequential** - 按优先级顺序执行
- **Parallel** - 所有 Hook 同时执行
- **Exclusive** - 只执行优先级最高的

### Hook 返回值

- **Continue** - 继续执行
- **Modified** - 修改数据后继续
- **Blocked** - 阻止执行
- **Skip** - 跳过
- **Replace** - 替换为其他操作

## 插件系统详解

### 插件类型

- **Capability** - 能力插件（工具、技能）
- **Hook** - Hook 插件（拦截、修改）
- **Provider** - 提供者插件（LLM、记忆）
- **Channel** - 通道插件（通信、API）

### 插件生命周期

1. 发现 → 2. 加载 → 3. 验证 → 4. 初始化 → 5. 执行 → 6. 卸载

### Slot 系统

Slot 是独占的，只有一个插件可以接管某个功能：

- DecisionEngine - 决策引擎
- MemoryProvider - 记忆提供者
- LLMProvider - LLM 提供者
- OutputFormatter - 输出格式化

### Capability 模型

插件注册到特定的能力类型：

- Decision - 决策能力
- Memory - 记忆能力
- Tool - 工具调用能力
- Weather - 天气查询
- Calendar - 日历操作
- 等等...

## 决策引擎详解

### 三种引擎

1. **RuleBasedEngine** - 规则驱动
2. **LLMEngine** - LLM 驱动
3. **HybridEngine** - 混合驱动

### 混合策略

- **RuleFirst** - 先尝试规则，失败时使用 LLM
- **LLMFirst** - 先尝试 LLM，失败时使用规则
- **Parallel** - 同时运行，选择置信度更高的
- **Sequential** - 先规则，然后用 LLM 增强

## 记忆系统详解

### 记忆类型

- **ShortTermMemory** - 短期记忆（容量有限）
- **LongTermMemory** - 长期记忆（持久化）
- **WorkingMemory** - 工作记忆（当前会话）

### 压缩策略

- **TimeBased** - 基于时间删除旧条目
- **CountBased** - 只保留最新的 N 条
- **ImportanceBased** - 删除低重要性条目
- **Hybrid** - 结合时间和重要性

## 使用示例

### 创建 Agent

```rust
use agent_pet_rs::prelude::*;

let config = AgentConfig::default();
let mut agent = Agent::new(config)?;
agent.start()?;
```

### 注册 Hook

```rust
agent.hooks_mut().register(
    HookPoint::OnInputReceived,
    100,
    Arc::new(|ctx| {
        println!("Input received!");
        Ok(HookResult::Continue)
    }),
)?;
```

### 触发 Hook

```rust
let ctx = HookContext::new(HookPoint::OnInputReceived, agent.id().to_string());
agent.hooks().trigger("on_input_received", &ctx)?;
```

### 使用决策引擎

```rust
let mut engine = RuleBasedEngine::new();
engine.add_rule(Rule { ... });

let context = DecisionContext { ... };
let decision = engine.decide(&context)?;
```

### 使用记忆系统

```rust
let mut memory = Memory::new(&config)?;

memory.store(MemoryEntry { ... })?;
let results = memory.search("query");
```
