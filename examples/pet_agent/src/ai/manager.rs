//! 提供商管理器

use std::collections::HashMap;
use std::path::PathBuf;
use super::error::AIError;
use super::provider::*;
use super::rate_limiter::RateLimiter;
use super::usage::UsageTracker;
use super::budget::BudgetTracker;

pub struct AIConfig {
    pub auto_switch: bool,
    pub switch_order: Vec<String>,
    pub providers: Vec<ProviderConfig>,
}

impl Default for AIConfig {
    fn default() -> Self {
        Self {
            auto_switch: true,
            switch_order: vec!["openrouter".to_string(), "openai".to_string(), "ollama".to_string()],
            providers: Vec::new(),
        }
    }
}

pub struct ProviderManager {
    providers: Vec<ProviderConfig>,
    rate_limiters: HashMap<String, RateLimiter>,
    usage_tracker: UsageTracker,
    budget_tracker: BudgetTracker,
    switch_order: Vec<String>,
    current_index: usize,
    auto_switch: bool,
}

impl ProviderManager {
    pub fn new(
        config: &AIConfig,
        usage_path: &PathBuf,
        budget_config: super::budget::BudgetConfig,
    ) -> Result<Self, AIError> {
        let mut rate_limiters = HashMap::new();
        for p in &config.providers {
            if p.enabled {
                rate_limiters.insert(p.name.clone(), RateLimiter::new(p.rate_limit.clone()));
            }
        }

        let usage_tracker = UsageTracker::load(usage_path)?;
        let budget_tracker = BudgetTracker::new(budget_config);

        Ok(Self {
            providers: config.providers.clone(),
            rate_limiters,
            usage_tracker,
            budget_tracker,
            switch_order: config.switch_order.clone(),
            current_index: 0,
            auto_switch: config.auto_switch,
        })
    }

    pub fn current_config(&self) -> Option<&ProviderConfig> {
        if self.switch_order.is_empty() {
            return None;
        }
        let name = &self.switch_order[self.current_index];
        self.providers.iter().find(|p| p.name == *name && p.enabled)
    }

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
            if let Some(config) = self.providers.iter().find(|p| p.name == *name) {
                if config.enabled {
                    return true;
                }
            }
        }
    }

    pub fn switch_provider(&mut self, name: &str) -> Result<(), AIError> {
        if let Some(index) = self.switch_order.iter().position(|n| n == name) {
            self.current_index = index;
            Ok(())
        } else {
            Err(AIError::UnknownProvider(name.to_string()))
        }
    }

    pub async fn chat_with_fallback(&mut self, messages: Vec<ChatMessage>) -> Result<String, AIError> {
        let start = self.current_index;
        loop {
            let config = match self.current_config() {
                Some(c) => c.clone(),
                None => return Err(AIError::NoProviderAvailable),
            };

            let name = config.name.clone();
            let estimated_tokens: u32 = messages.iter().map(|m| (m.content.len() / 4) as u32).sum();

            if let Some(rl) = self.rate_limiters.get_mut(&name) {
                if let Err(e) = rl.wait_if_needed(estimated_tokens).await {
                    eprintln!("Provider {} 速率限制: {}", name, e);
                    if !self.switch_to_next() { return Err(e); }
                    continue;
                }
            }

            match super::adapters::chat(messages.clone(), &config).await {
                Ok(response) => {
                    self.budget_tracker.check_budget(response.usage.total_tokens as f64 / 1000000.0 * 0.01).ok();
                    self.usage_tracker.record(&name, &config.model, &response.usage);
                    return Ok(response.content);
                }
                Err(e) => {
                    eprintln!("Provider {} 失败: {}", name, e);
                    if !self.switch_to_next() { return Err(e); }
                    if self.current_index == start { return Err(AIError::AllProvidersFailed); }
                }
            }
        }
    }

    pub fn current_provider_name(&self) -> &str {
        if self.switch_order.is_empty() { return ""; }
        &self.switch_order[self.current_index]
    }

    pub fn usage_stats(&self) -> &super::usage::UsageStats {
        self.usage_tracker.stats()
    }

    pub fn usage_summary(&self) -> String {
        self.usage_tracker.summary()
    }

    pub fn export_usage_json(&self) -> Result<String, AIError> {
        self.usage_tracker.export_json()
    }

    pub fn export_usage_csv(&self) -> String {
        self.usage_tracker.export_csv()
    }

    pub fn rate_limit_status(&self) -> String {
        let name = self.current_provider_name();
        if let Some(rl) = self.rate_limiters.get(name) {
            let (rm, rh, tm, th) = rl.get_status();
            format!("请求: {}/min | {}/h  Tokens: {}k/min | {}k/h", rm, rh, tm/1000, th/1000)
        } else {
            "不限速".to_string()
        }
    }
}
