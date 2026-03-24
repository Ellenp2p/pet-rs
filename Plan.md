# agent-pet-rs 框架架构重构计划

## 项目概述

agent-pet-rs 是一个用于构建智能虚拟生命的 Rust 框架，支持 WASM 插件系统。本次重构目标是将框架核心与渲染层分离，使框架可在任何渲染系统中使用。

## 架构设计

### 核心原则

1. **核心框架（src/）** - 纯 Rust 实现，不依赖任何渲染库
2. **渲染层（examples/）** - 使用 Bevy、ratatui 或其他渲染系统
3. **适配器模式** - 在 examples/ 中定义渲染系统的适配器

### 目录结构

```
src/                        # 纯 Rust 核心框架（无 Bevy 依赖）
├── config.rs              # 配置管理
├── dependency.rs          # 依赖解析
├── error.rs               # 错误类型
├── hooks.rs               # 钩子系统
├── network.rs             # 网络通道
├── permission.rs          # 权限管理
├── wasm/
│   ├── plugin_trait.rs    # 插件 trait
│   └── wasmtime_loader.rs # WASM 加载器
├── lib.rs                 # 框架入口
└── prelude.rs             # 统一导出

examples/                   # 渲染层（可选 Bevy 或 crossterm）
├── bevy_adapter.rs        # Bevy 适配器
├── basic_pet.rs           # Bevy 示例
├── cli_pet.rs             # CLI/TUI 示例
├── wasm_*/                # WASM 插件
└── config.json            # 配置文件
```

## 重构计划

### 阶段 1：移除 src/ 中的 Bevy 依赖

#### 1.1 修改 src/lib.rs
- [x] 移除 `use bevy::prelude::*`
- [x] 移除 `FrameworkSet` 枚举（SystemSet）
- [x] 移除 `FrameworkPlugin` 结构体（Plugin trait）
- [x] 移除 `configure_backend` 函数
- [x] 保留核心模块导出

#### 1.2 修改 src/config.rs
- [x] 移除 `#[derive(Resource)]` 宏
- [x] 移除 `use bevy::prelude::*`
- [x] 保持纯 Rust 实现

#### 1.3 修改 src/hooks/mod.rs
- [x] 移除 `#[derive(Resource)]` 宏
- [x] 移除 `use bevy::prelude::*`
- [x] 保持纯 Rust 实现

#### 1.4 修改 src/network/mod.rs
- [x] 移除 `#[derive(Resource)]` 宏
- [x] 移除 `use bevy::prelude::*`
- [x] 保持纯 Rust 实现

#### 1.5 修改 src/wasm/bridge.rs
- [x] 移除 `#[derive(Resource)]` 宏
- [x] 移除 `use bevy::prelude::*`
- [x] 保持纯 Rust 实现

#### 1.6 删除 src/plugins/wasm_plugin.rs
- [x] 删除整个文件（移到 examples/）

### 阶段 2：创建 Bevy 适配器

#### 2.1 创建 examples/bevy_adapter.rs
- [x] 定义 Bevy Resource 包装器
- [x] 实现 FrameworkPlugin
- [x] 实现 WasmPluginBevy
- [x] 提供核心框架到 Bevy 的转换

#### 2.2 更新 examples/basic_pet.rs
- [x] 导入 bevy_adapter.rs
- [x] 使用新的 Resource 包装器
- [x] 保持功能不变

### 阶段 3：创建 CLI/TUI 示例

#### 3.1 创建 examples/cli_pet.rs
- [x] 使用 crossterm 实现 TUI 界面
- [x] 调用核心框架 API
- [x] 验证框架通用性

#### 3.2 更新 Cargo.toml
- [x] 添加 crossterm 依赖（可选）
- [x] 添加 cli_pet 示例

### 阶段 4：测试验证

#### 4.1 编译测试
- [x] 验证 src/ 独立编译
- [x] 验证 basic_pet 示例编译
- [x] 验证 cli_pet 示例编译

#### 4.2 功能测试
- [x] 运行所有单元测试
- [x] 验证 Bevy 示例功能
- [x] 验证 CLI 示例功能

#### 4.3 最终验证
- [x] 运行 `cargo check`
- [x] 运行 `cargo test`
- [x] 运行 `cargo clippy`
- [x] 运行 `cargo fmt --check`

## 预期效果

1. **核心框架** - 纯 Rust 实现，可在任何项目中使用
2. **渲染层分离** - Bevy、crossterm 或其他渲染系统可自由选择
3. **向后兼容** - 现有 Bevy 示例继续工作
4. **易于测试** - 核心逻辑与渲染逻辑分离

## 验证命令

```bash
# 检查核心框架编译
cargo check

# 检查 Bevy 示例
cargo check --example basic_pet --features wasm-plugin

# 检查 CLI 示例
cargo check --example cli_pet --features wasm-plugin

# 运行测试
cargo test

# 运行 lint
cargo clippy

# 检查格式
cargo fmt --check
```
src/                        # 纯 Rust 核心框架（无 Bevy 依赖）
├── config.rs              # 配置管理
├── dependency.rs          # 依赖解析
├── error.rs               # 错误类型
├── hooks.rs               # 钩子系统
├── network.rs             # 网络通道
├── permission.rs          # 权限管理
├── wasm/
│   ├── plugin_trait.rs    # 插件 trait
│   └── wasmtime_loader.rs # WASM 加载器
├── lib.rs                 # 框架入口
└── prelude.rs             # 统一导出

examples/                   # 渲染层（可选 Bevy 或 ratatui）
├── bevy_adapter.rs        # Bevy 适配器
├── basic_pet.rs           # Bevy 示例
├── cli_pet.rs             # CLI/TUI 示例
├── wasm_*/                # WASM 插件
└── config.json            # 配置文件
```

## 重构计划

### 阶段 1：移除 src/ 中的 Bevy 依赖

#### 1.1 修改 src/lib.rs
- [x] 移除 `use bevy::prelude::*`
- [x] 移除 `FrameworkSet` 枚举（SystemSet）
- [x] 移除 `FrameworkPlugin` 结构体（Plugin trait）
- [x] 移除 `configure_backend` 函数
- [x] 保留核心模块导出

#### 1.2 修改 src/config.rs
- [x] 移除 `#[derive(Resource)]` 宏
- [x] 移除 `use bevy::prelude::*`
- [x] 保持纯 Rust 实现

#### 1.3 修改 src/hooks/mod.rs
- [x] 移除 `#[derive(Resource)]` 宏
- [x] 移除 `use bevy::prelude::*`
- [x] 保持纯 Rust 实现

#### 1.4 修改 src/network/mod.rs
- [x] 移除 `#[derive(Resource)]` 宏
- [x] 移除 `use bevy::prelude::*`
- [x] 保持纯 Rust 实现

#### 1.5 修改 src/wasm/bridge.rs
- [x] 移除 `#[derive(Resource)]` 宏
- [x] 移除 `use bevy::prelude::*`
- [x] 保持纯 Rust 实现

#### 1.6 删除 src/plugins/wasm_plugin.rs
- [x] 删除整个文件（移到 examples/）

### 阶段 2：创建 Bevy 适配器

#### 2.1 创建 examples/bevy_adapter.rs
- [x] 定义 Bevy Resource 包装器
- [x] 实现 FrameworkPlugin
- [x] 实现 WasmPluginBevy
- [x] 提供核心框架到 Bevy 的转换

#### 2.2 更新 examples/basic_pet.rs
- [x] 导入 bevy_adapter.rs
- [x] 使用新的 Resource 包装器
- [x] 保持功能不变

### 阶段 3：创建 CLI/TUI 示例

#### 3.1 创建 examples/cli_pet.rs
- [ ] 使用 ratatui 实现 TUI 界面
- [ ] 调用核心框架 API
- [ ] 验证框架通用性

#### 3.2 更新 Cargo.toml
- [ ] 添加 ratatui 依赖
- [ ] 添加 crossterm 依赖
- [ ] 添加 cli_pet 示例

### 阶段 4：测试验证

#### 4.1 编译测试
- [ ] 验证 src/ 独立编译
- [ ] 验证 basic_pet 示例编译
- [ ] 验证 cli_pet 示例编译

#### 4.2 功能测试
- [ ] 运行所有单元测试
- [ ] 验证 Bevy 示例功能
- [ ] 验证 CLI 示例功能

#### 4.3 最终验证
- [ ] 运行 `cargo check`
- [ ] 运行 `cargo test`
- [ ] 运行 `cargo clippy`
- [ ] 运行 `cargo fmt --check`

## 预期效果

1. **核心框架** - 纯 Rust 实现，可在任何项目中使用
2. **渲染层分离** - Bevy、ratatui 或其他渲染系统可自由选择
3. **向后兼容** - 现有 Bevy 示例继续工作
4. **易于测试** - 核心逻辑与渲染逻辑分离

## 验证命令

```bash
# 检查核心框架编译
cargo check

# 检查 Bevy 示例
cargo check --example basic_pet --features wasm-plugin

# 检查 CLI 示例
cargo check --example cli_pet --features wasm-plugin

# 运行测试
cargo test

# 运行 lint
cargo clippy

# 检查格式
cargo fmt --check
```
