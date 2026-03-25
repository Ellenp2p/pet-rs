//! Hook 系统模块
//!
//! 提供 28 个 Hook 点的定义、注册和执行。

pub mod context;
pub mod points;
pub mod registry;
pub mod runner;

pub use context::HookContext;
pub use points::HookPoint;
pub use registry::HookRegistry;
pub use runner::HookRunner;
