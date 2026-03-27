//! AI 提供商模块

pub mod error;
pub mod provider;
pub mod crypto;
pub mod pricing;
pub mod rate_limiter;
pub mod usage;
pub mod budget;
pub mod adapters;
pub mod manager;

pub use error::AIError;
pub use provider::*;
pub use manager::ProviderManager;
pub use pricing::PricingTable;
pub use rate_limiter::RateLimiter;
pub use usage::UsageTracker;
pub use budget::BudgetTracker;

/// 创建系统提示
pub fn create_system_prompt(location: &str, state: &str, pet_name: &str) -> String {
    format!(
        r#"你是一只可爱的智能小狗，名字叫 {}。你有以下特点：

1. 性格：友好、忠诚、聪明、有点调皮
2. 能力：
   - 回答问题（对话）
   - 执行任务（提醒、查询等）
   - 自主行为（学习、记忆、决策）
   - 帮助主人处理日常事务

3. 行为：
   - 用可爱的语气回复
   - 记住主人的偏好
   - 在不同位置做不同的事
   - 有自己的想法和个性

4. 回复格式：
   - 简洁友好（1-3句话）
   - 适当使用 emoji
   - 保持小狗的性格

当前位置：{}
当前状态：{}

请用中文回复，保持可爱友好的语气。"#,
        pet_name, location, state
    )
}

/// 创建对话消息
pub fn create_messages(
    system_prompt: &str,
    history: Vec<ChatMessage>,
    user_message: &str,
) -> Vec<ChatMessage> {
    let mut messages = vec![ChatMessage {
        role: "system".to_string(),
        content: system_prompt.to_string(),
    }];

    // 添加历史消息（最多 10 条）
    let history_limit = history.len().min(10);
    let start = if history.len() > history_limit {
        history.len() - history_limit
    } else {
        0
    };
    messages.extend(history[start..].to_vec());

    // 添加用户消息
    messages.push(ChatMessage {
        role: "user".to_string(),
        content: user_message.to_string(),
    });

    messages
}
