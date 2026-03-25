//! 通信层模块
//!
//! 提供通道管理、消息定义和消息路由。

pub mod channel;
pub mod message;
pub mod router;

pub use channel::Channel;
pub use message::Message;
pub use router::MessageRouter;
