//! AI 提供商管理器

use std::collections::HashMap;

use super::adapters::{AnthropicProvider, OpenAIProvider};
use super::budget::{BudgetConfig, BudgetStatus, BudgetTracker};
use super::error::AIError;
use super::provider::*;
use super::rate_limiter::RateLimiter;
use super::usage::UsageTracker;

/// AI 配置
#[derive(Debug, Clone)]
pub struct AIConfig {
    pub default_provider: String,
    pub auto_switch: bool,
    pub switch_order: Vec<String>,
    pub providers: HashMap<String, ProviderConfig>,
    pub budget: BudgetConfig,
}

impl Default for AIConfig {
    fn default() -> Self {
        Self {
            default_provider: "openai".to_string(),
            auto_switch: true,
            switch_order: vec!["openai".to_string(), "anthropic".to_string()],
            providers: HashMap::new(),
            budget: BudgetConfig::default(),
        }
    }
}

/// 提供商管理器
pub struct AIProviderManager {
    /// 内置提供商
    builtin_providers: HashMap<String, Box<dyn AIProvider>>,
    /// 提供商配置
    configs: HashMap<String, ProviderConfig>,
    /// 速率限制器
    rate_limiters: HashMap<String, RateLimiter>,
    /// 使用量追踪器
    usage_tracker: UsageTracker,
    /// 预算追踪器
    budget_tracker: BudgetTracker,
    /// 切换顺序
    switch_order: Vec<String>,
    /// 当前提供商索引
    current_index: usize,
    /// 是否自动切换
    auto_switch: bool,
}

impl AIProviderManager {
    /// 创建新的管理器
    pub fn new(config: &AIConfig) -> Result<Self, AIError> {
        let mut builtin_providers: HashMap<String, Box<dyn AIProvider>> = HashMap::new();
        let mut rate_limiters = HashMap::new();

        // 注册内置提供商
        builtin_providers.insert("openai".to_string(), Box::new(OpenAIProvider::new()));
        builtin_providers.insert("anthropic".to_string(), Box::new(AnthropicProvider::new()));

        // 为每个启用的提供商创建速率限制器
        for (name, provider_config) in &config.providers {
            if provider_config.enabled {
                rate_limiters.insert(
                    name.clone(),
                    RateLimiter::new(provider_config.rate_limit.clone()),
                );
            }
        }

        Ok(Self {
            builtin_providers,
            configs: config.providers.clone(),
            rate_limiters,
            usage_tracker: UsageTracker::new(),
            budget_tracker: BudgetTracker::new(config.budget.clone()),
            switch_order: config.switch_order.clone(),
            current_index: 0,
            auto_switch: config.auto_switch,
        })
    }

    /// 获取当前提供商配置
    pub fn current_config(&self) -> Option<&ProviderConfig> {
        if self.switch_order.is_empty() {
            return None;
        }
        let name = &self.switch_order[self.current_index];
        self.configs.get(name)
    }

    /// 获取当前提供商名称
    pub fn current_provider_name(&self) -> &str {
        if self.switch_order.is_empty() {
            return "";
        }
        &self.switch_order[self.current_index]
    }

    /// 切换到下一个提供商
    pub fn switch_to_next(&mut self) -> bool {
        if !self.auto_switch || self.switch_order.is_empty() {
            return false;
        }
        let start = self.current_index;
        loop {
            self.current_index = (self.current_index + 1) % self.switch_order.len();
            if self.current_index == start {
                return false;
            }
            let name = &self.switch_order[self.current_index];
            if let Some(config) = self.configs.get(name) {
                if config.enabled {
                    return true;
                }
            }
        }
    }

    /// 手动切换提供商
    pub fn switch_provider(&mut self, name: &str) -> Result<(), AIError> {
        if let Some(index) = self.switch_order.iter().position(|n| n == name) {
            self.current_index = index;
            Ok(())
        } else {
            Err(AIError::UnknownProvider(name.to_string()))
        }
    }

    /// 聊天（带自动切换）
    pub fn chat(&mut self, messages: Vec<ChatMessage>) -> Result<ChatResponse, AIError> {
        let start = self.current_index;

        loop {
            let provider_name = self.current_provider_name().to_string();
            let config = match self.configs.get(&provider_name) {
                Some(c) => c.clone(),
                None => return Err(AIError::NoProviderAvailable),
            };

            // 检查预算
            if self.budget_tracker.check_and_record(0.0) == BudgetStatus::Exceeded {
                return Err(AIError::BudgetExceeded("预算已超限".to_string()));
            }

            // 获取提供商
            let provider = match self.builtin_providers.get(&provider_name) {
                Some(p) => p,
                None => {
                    if !self.switch_to_next() {
                        return Err(AIError::UnknownProvider(provider_name));
                    }
                    continue;
                }
            };

            // 检查速率限制
            if let Some(rl) = self.rate_limiters.get_mut(&provider_name) {
                let estimated_tokens: u32 =
                    messages.iter().map(|m| (m.content.len() / 4) as u32).sum();
                if let Err(e) = rl.check_request(estimated_tokens) {
                    if !self.switch_to_next() {
                        return Err(e);
                    }
                    continue;
                }
            }

            // 执行聊天
            match provider.chat(messages.clone(), &config) {
                Ok(response) => {
                    // 计算费用
                    let cost = provider.calculate_cost(&response.usage, &config);

                    // 记录使用量
                    self.usage_tracker
                        .record(&provider_name, &config.model, &response.usage, cost);

                    // 更新预算
                    self.budget_tracker.check_and_record(cost);

                    return Ok(response);
                }
                Err(e) => {
                    if !self.switch_to_next() || self.current_index == start {
                        return Err(e);
                    }
                }
            }
        }
    }

    /// 流式聊天
    pub fn chat_stream(
        &mut self,
        messages: Vec<ChatMessage>,
        on_chunk: Box<dyn Fn(String) + Send>,
    ) -> Result<ChatResponse, AIError> {
        let provider_name = self.current_provider_name().to_string();
        let config = self
            .configs
            .get(&provider_name)
            .ok_or_else(|| AIError::UnknownProvider(provider_name.clone()))?
            .clone();

        let provider = self
            .builtin_providers
            .get(&provider_name)
            .ok_or_else(|| AIError::UnknownProvider(provider_name.clone()))?;

        let response = provider.chat_stream(messages, &config, on_chunk)?;

        // 计算费用
        let cost = provider.calculate_cost(&response.usage, &config);

        // 记录使用量
        self.usage_tracker
            .record(&provider_name, &config.model, &response.usage, cost);

        // 更新预算
        self.budget_tracker.check_and_record(cost);

        Ok(response)
    }

    /// 列出所有提供商
    pub fn list_providers(&self) -> Vec<String> {
        self.switch_order.clone()
    }

    /// 获取使用量统计
    pub fn usage_stats(&self) -> &UsageTracker {
        &self.usage_tracker
    }

    /// 获取使用量摘要
    pub fn usage_summary(&self) -> String {
        self.usage_tracker.summary()
    }

    /// 导出使用量 JSON
    pub fn export_usage_json(&self) -> Result<String, AIError> {
        self.usage_tracker
            .export_json()
            .map_err(|e| AIError::ParseError(e.to_string()))
    }

    /// 导出使用量 CSV
    pub fn export_usage_csv(&self) -> String {
        self.usage_tracker.export_csv()
    }

    /// 获取速率限制状态
    pub fn rate_limit_status(&self) -> String {
        let name = self.current_provider_name();
        if let Some(rl) = self.rate_limiters.get(name) {
            let (rm, rh, tm, th) = rl.get_status();
            format!(
                "请求: {}/min | {}/h  Tokens: {}k/min | {}k/h",
                rm,
                rh,
                tm / 1000,
                th / 1000
            )
        } else {
            "不限速".to_string()
        }
    }

    /// 注册自定义提供商
    pub fn register_provider(
        &mut self,
        name: String,
        provider: Box<dyn AIProvider>,
        config: ProviderConfig,
    ) {
        self.builtin_providers.insert(name.clone(), provider);
        self.configs.insert(name.clone(), config);
        if !self.switch_order.contains(&name) {
            self.switch_order.push(name);
        }
    }

    /// 注销提供商
    pub fn unregister_provider(&mut self, name: &str) {
        self.builtin_providers.remove(name);
        self.configs.remove(name);
        self.rate_limiters.remove(name);
        self.switch_order.retain(|n| n != name);
    }
}
