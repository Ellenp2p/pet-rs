//! 上下文窗口
//!
//! 管理上下文的窗口大小和截断。

use super::context_impl::Context;
use crate::error::FrameworkError;

/// 上下文窗口配置
pub struct ContextWindowConfig {
    /// 最大历史记录数
    pub max_history: usize,
    /// 最大 token 数（近似）
    pub max_tokens: usize,
}

impl Default for ContextWindowConfig {
    fn default() -> Self {
        Self {
            max_history: 100,
            max_tokens: 4000,
        }
    }
}

/// 上下文窗口
pub struct ContextWindow {
    /// 配置
    config: ContextWindowConfig,
}

impl ContextWindow {
    /// 创建新的上下文窗口
    pub fn new(config: ContextWindowConfig) -> Self {
        Self { config }
    }

    /// 应用窗口限制
    pub fn apply(&self, context: &mut Context) -> Result<(), FrameworkError> {
        // 限制历史记录数
        if context.history.len() > self.config.max_history {
            let drain_count = context.history.len() - self.config.max_history;
            context.history.drain(0..drain_count);
        }

        // 估算 token 数并截断
        let mut token_count = 0;
        let mut truncate_index = 0;

        for (i, entry) in context.history.iter().enumerate() {
            // 简单的 token 估算：1 token ≈ 4 字符
            let entry_tokens = entry.content.len() / 4;
            token_count += entry_tokens;

            if token_count > self.config.max_tokens {
                truncate_index = i;
                break;
            }
        }

        if truncate_index > 0 {
            context.history.drain(0..truncate_index);
        }

        Ok(())
    }

    /// 获取当前 token 数（近似）
    pub fn estimate_tokens(&self, context: &Context) -> usize {
        context.history.iter().map(|e| e.content.len() / 4).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_window() {
        let config = ContextWindowConfig {
            max_history: 2,
            max_tokens: 100,
        };
        let window = ContextWindow::new(config);

        let mut context = Context::new();
        context.add_history("user".to_string(), "hello".to_string());
        context.add_history("assistant".to_string(), "hi".to_string());
        context.add_history("user".to_string(), "how are you?".to_string());

        window.apply(&mut context).unwrap();
        assert_eq!(context.history_len(), 2);
    }
}
