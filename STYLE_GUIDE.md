# pet-rs Style Guide

简短规范（团队/仓库级）：

- 格式化：统一使用 `rustfmt`（仓库包含 `rustfmt.toml`），本地运行 `cargo fmt`。CI 要求 `cargo fmt -- --check` 通过。
- 代码风格：遵循 Rust 默认风格（命名：`snake_case` 函数/变量，`CamelCase` 类型/结构体，常量 `SCREAMING_SNAKE`）。
- 日志：使用 `log` crate（`env_logger` 在开发中启用），不在库中直接打印到 stdout/stderr 除非调试。
- 不要提交生成的二进制文件到仓库；资源（如 `assets/`）可以包含示例/占位文件，但大型媒体请使用 release 或外部资源。
- 模块组织：每个模块放在 `src/` 下，平台特定实现放在 `src/platform/` 并通过条件编译导出。
- 测试：尽量为资源加载、帧切换、alpha 判定写单元/集成测试，使用 `cargo test` 运行。
- 提交信息：简洁明了，首行不超过 72 个字符，使用祈使句（例：`Add macOS platform placeholder`）。

常用命令：
- 格式化：`cargo fmt`
- 构建：`cargo build --release`
- 运行示例生成器：`cargo run --release -- --gen-spritesheet`
- 运行测试：`cargo test --all --verbose`
