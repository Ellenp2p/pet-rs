//! Agent 核心模块
//!
//! 提供 Agent 的核心结构、主循环、人格系统和角色系统。

pub mod core;
pub mod loop_impl;
pub mod personality;
pub mod role;

pub use core::Agent;
pub use loop_impl::AgentLoop;
pub use personality::Personality;
pub use role::Role;
