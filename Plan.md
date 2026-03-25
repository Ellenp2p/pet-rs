# agent-pet-rs 2.0 - WASM Agent 框架实施计划

## 项目概述

agent-pet-rs 是一个基于 WASM 插件的智能 Agent 框架，支持角色扮演、自主行为和插件扩展。

## 核心特性

- 🎭 **角色扮演** - Agent 可以有不同的性格和角色
- 🤖 **自主行为** - Agent 可以自主决策和执行任务
- 🔌 **插件扩展** - 通过 WASM 插件扩展能力
- 🧠 **可选 LLM** - 支持 LLM 但不强制依赖
- ⚡ **高性能** - Rust + WASM 带来的高性能
- 🎮 **多场景** - 游戏角色、助手、自动化工具

## 当前状态

**版本**: 2.0.0-alpha
**完成度**: Phase 1-5 完成 (100% 核心功能)

## 架构设计

### 核心组件

1. **Agent Core** - Agent 核心结构和主循环 ✅
2. **Hook System** - 28 个 Hook 点的系统 ✅
3. **Plugin System** - WASM 插件管理 ✅
4. **Decision Engine** - 决策引擎（规则/LLM/混合）✅
5. **Memory System** - 记忆系统（含压缩和持久化）✅
6. **Context Manager** - 上下文管理 ✅

### 目录结构

```
src/
├── agent/              # Agent 核心 ✅
│   ├── mod.rs
│   ├── core.rs
│   ├── loop_impl.rs
│   ├── personality.rs
│   └── role.rs
├── hooks/              # Hook 系统 ✅
│   ├── mod.rs
│   ├── points.rs
│   ├── runner.rs
│   ├── registry.rs
│   └── context.rs
├── plugins/            # 插件系统 (Phase 2)
│   ├── mod.rs
│   ├── loader.rs
│   ├── discovery.rs
│   ├── validator.rs
│   ├── lifecycle.rs
│   ├── slots.rs
│   ├── capabilities.rs
│   └── manifest.rs
├── wasm/               # WASM 系统 (已有基础)
│   ├── mod.rs
│   ├── abi.rs
│   ├── wasmtime_loader.rs
│   ├── sandbox.rs
│   └── host_functions.rs
├── decision/           # 决策引擎 ✅
│   ├── mod.rs
│   ├── engine.rs
│   ├── rule_based.rs
│   ├── llm_based.rs
│   └── hybrid.rs
├── memory/             # 记忆系统 ✅
│   ├── mod.rs
│   ├── memory_impl.rs
│   ├── short_term.rs
│   ├── long_term.rs
│   ├── working.rs
│   └── compaction.rs
├── context/            # 上下文管理 ✅
│   ├── mod.rs
│   ├── context_impl.rs
│   ├── builder.rs
│   └── window.rs
├── communication/      # 通信层 ✅
│   ├── mod.rs
│   ├── channel.rs
│   ├── message.rs
│   └── router.rs
├── config.rs
├── dependency.rs
├── error.rs
├── permission.rs
├── lib.rs
└── prelude.rs
```

## 实施阶段

### Phase 1: 核心框架 ✅ 完成
- [x] 创建目录结构
- [x] 实现 Agent 核心结构
- [x] 实现 28 个 Hook 点
- [x] 实现 Hook 注册和执行器
- [x] 实现优先级系统
- [x] 编写测试 (59 个测试全部通过)

### Phase 2: WASM 插件系统 ✅ 完成
- [x] 定义完整 WASM ABI
- [x] 实现插件加载/卸载
- [x] 实现沙箱安全
- [x] 实现热重载
- [x] 实现宿主函数
- [x] 实现 Slot 系统
- [x] 实现 Capability 模型
- [x] 实现生命周期管理
- [x] 实现插件发现
- [x] 编写测试

### Phase 3: 决策引擎 ✅ 完成
- [x] 实现规则引擎
- [x] 实现 LLM 引擎（可选）
- [x] 实现混合引擎
- [x] 实现决策上下文
- [x] 编写测试

### Phase 4: 记忆系统 ✅ 完成
- [x] 实现短期记忆
- [x] 实现长期记忆
- [x] 实现工作记忆
- [x] 实现记忆压缩
- [x] 实现记忆持久化
- [x] 编写测试

### Phase 5: 插件管理 ✅ 完成
- [x] 实现 Slot 系统
- [x] 实现 Capability 模型
- [x] 实现生命周期管理
- [x] 实现插件发现
- [x] 编写测试
- [ ] 编写测试

### Phase 6: 示例和文档 ✅ 完成
- [x] 创建智能助手示例
- [x] 创建 Hook 系统示例
- [x] 创建记忆系统示例
- [x] 创建决策引擎示例
- [x] 编写架构文档
- [ ] 编写 API 文档
- [ ] 编写教程

### Phase 7: 测试和优化 🔄 进行中
- [x] 集成测试
- [ ] 性能测试
- [ ] 安全测试
- [ ] 文档完善

## Hook 系统设计 (28 个 Hook 点)

### 输入处理层 (3 个)
1. `on_input_received` - 输入到达时
2. `before_input_parse` - 解析输入前
3. `after_input_parse` - 解析输入后

### 上下文构建层 (4 个)
4. `before_context_build` - 构建上下文前
5. `after_context_build` - 构建上下文后
6. `before_memory_load` - 加载记忆前
7. `after_memory_load` - 加载记忆后

### 决策层 (4 个)
8. `before_decision` - 决策前
9. `after_decision` - 决策后
10. `before_llm_call` - LLM 调用前
11. `after_llm_call` - LLM 调用后

### 动作执行层 (4 个)
12. `before_action` - 动作执行前
13. `after_action` - 动作执行后
14. `before_tool_call` - 工具调用前
15. `after_tool_call` - 工具调用后

### 输出生成层 (3 个)
16. `before_output` - 生成输出前
17. `after_output` - 生成输出后
18. `before_response` - 发送响应前

### 记忆管理层 (3 个)
19. `before_memory_write` - 写入记忆前
20. `after_memory_write` - 写入记忆后
21. `before_memory_compact` - 记忆压缩前

### 角色/人格层 (3 个)
22. `before_role_apply` - 应用角色前
23. `after_role_apply` - 应用角色后
24. `on_personality_change` - 人格变化时

### 生命周期层 (4 个)
25. `on_agent_start` - Agent 启动时
26. `on_agent_stop` - Agent 停止时
27. `on_session_start` - 会话开始时
28. `on_session_end` - 会话结束时

## 技术选型

| 组件 | 选择 |
|------|------|
| WASM 运行时 | wasmtime 24 |
| LLM 客户端 | reqwest + 自定义 |
| 序列化 | serde + serde_json |
| 异步运行时 | tokio |
| 日志 | tracing |
| 错误处理 | thiserror + anyhow |
