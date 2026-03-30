//! 使用量追踪

use super::provider::TokenUsage;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 使用量记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageRecord {
    pub timestamp: u64,
    pub provider: String,
    pub model: String,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub cost: f64,
}

/// 提供商统计
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProviderStats {
    pub requests: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cost: f64,
}

/// 模型统计
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelStats {
    pub requests: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cost: f64,
}

/// 使用量统计
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UsageStats {
    pub total_requests: u64,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_cost: f64,
    pub by_provider: HashMap<String, ProviderStats>,
    pub by_model: HashMap<String, ModelStats>,
}

/// 使用量追踪器
pub struct UsageTracker {
    records: Vec<UsageRecord>,
    stats: UsageStats,
}

impl Default for UsageTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl UsageTracker {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
            stats: UsageStats::default(),
        }
    }

    pub fn record(&mut self, provider: &str, model: &str, usage: &TokenUsage, cost: f64) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let record = UsageRecord {
            timestamp: now,
            provider: provider.to_string(),
            model: model.to_string(),
            input_tokens: usage.input_tokens,
            output_tokens: usage.output_tokens,
            cost,
        };

        self.update_stats(&record);
        self.records.push(record);
    }

    fn update_stats(&mut self, record: &UsageRecord) {
        self.stats.total_requests += 1;
        self.stats.total_input_tokens += record.input_tokens as u64;
        self.stats.total_output_tokens += record.output_tokens as u64;
        self.stats.total_cost += record.cost;

        let ps = self
            .stats
            .by_provider
            .entry(record.provider.clone())
            .or_default();
        ps.requests += 1;
        ps.input_tokens += record.input_tokens as u64;
        ps.output_tokens += record.output_tokens as u64;
        ps.cost += record.cost;

        let ms = self.stats.by_model.entry(record.model.clone()).or_default();
        ms.requests += 1;
        ms.input_tokens += record.input_tokens as u64;
        ms.output_tokens += record.output_tokens as u64;
        ms.cost += record.cost;
    }

    pub fn stats(&self) -> &UsageStats {
        &self.stats
    }

    pub fn summary(&self) -> String {
        format!(
            "请求: {} | Tokens: {}k | 费用: ${:.4}",
            self.stats.total_requests,
            (self.stats.total_input_tokens + self.stats.total_output_tokens) / 1000,
            self.stats.total_cost
        )
    }

    pub fn export_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.stats)
    }

    pub fn export_csv(&self) -> String {
        let mut csv = String::from("timestamp,provider,model,input_tokens,output_tokens,cost\n");
        for r in &self.records {
            csv.push_str(&format!(
                "{},{},{},{},{},{:.6}\n",
                r.timestamp, r.provider, r.model, r.input_tokens, r.output_tokens, r.cost
            ));
        }
        csv
    }

    pub fn records(&self) -> &[UsageRecord] {
        &self.records
    }
}
