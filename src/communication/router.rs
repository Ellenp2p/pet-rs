//! 消息路由器
//!
//! 负责将消息路由到正确的处理器。

use crate::error::FrameworkError;
use std::collections::HashMap;

use super::message::Message;

/// 消息处理器 trait
pub trait MessageHandler {
    /// 处理消息
    fn handle(&self, message: Message) -> Result<Option<Message>, FrameworkError>;
}

/// 消息路由器
pub struct MessageRouter {
    /// 处理器映射
    handlers: HashMap<String, Box<dyn MessageHandler>>,
    /// 默认处理器
    default_handler: Option<Box<dyn MessageHandler>>,
}

impl MessageRouter {
    /// 创建新的消息路由器
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            default_handler: None,
        }
    }

    /// 注册处理器
    pub fn register(&mut self, name: String, handler: Box<dyn MessageHandler>) {
        self.handlers.insert(name, handler);
    }

    /// 设置默认处理器
    pub fn set_default_handler(&mut self, handler: Box<dyn MessageHandler>) {
        self.default_handler = Some(handler);
    }

    /// 路由消息
    pub fn route(&self, message: Message) -> Result<Option<Message>, FrameworkError> {
        // 根据发送者查找处理器
        if let Some(handler) = self.handlers.get(&message.sender) {
            return handler.handle(message);
        }

        // 使用默认处理器
        if let Some(handler) = &self.default_handler {
            return handler.handle(message);
        }

        // 没有处理器，返回原始消息
        Ok(Some(message))
    }
}

impl Default for MessageRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestHandler;

    impl MessageHandler for TestHandler {
        fn handle(&self, message: Message) -> Result<Option<Message>, FrameworkError> {
            Ok(Some(Message::new(
                "system".to_string(),
                format!("Handled: {}", message.content),
            )))
        }
    }

    #[test]
    fn test_message_router() {
        let mut router = MessageRouter::new();
        router.register("test".to_string(), Box::new(TestHandler));

        let message = Message::new("test".to_string(), "hello".to_string());
        let result = router.route(message).unwrap();
        assert!(result.is_some());
    }
}
