//! 预算管理

use serde::{Deserialize, Serialize};

/// 预算配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetConfig {
    pub enabled: bool,
    pub daily_limit: Option<f64>,
    pub monthly_limit: Option<f64>,
    pub warning_threshold: f64,
}

impl Default for BudgetConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            daily_limit: None,
            monthly_limit: None,
            warning_threshold: 0.8,
        }
    }
}

/// 预算状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BudgetStatus {
    /// 正常
    Ok,
    /// 警告（接近限额）
    Warning,
    /// 超限
    Exceeded,
}

/// 预算追踪器
pub struct BudgetTracker {
    config: BudgetConfig,
    daily_spent: f64,
    monthly_spent: f64,
    daily_reset: u64,
    monthly_reset: u64,
}

impl BudgetTracker {
    pub fn new(config: BudgetConfig) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self {
            config,
            daily_spent: 0.0,
            monthly_spent: 0.0,
            daily_reset: now + 86400,
            monthly_reset: now + 2592000,
        }
    }

    /// 检查并记录费用，返回预算状态
    pub fn check_and_record(&mut self, cost: f64) -> BudgetStatus {
        if !self.config.enabled {
            return BudgetStatus::Ok;
        }

        self.reset_if_needed();

        // 检查是否超限
        if let Some(daily) = self.config.daily_limit {
            if self.daily_spent + cost > daily {
                return BudgetStatus::Exceeded;
            }
        }

        if let Some(monthly) = self.config.monthly_limit {
            if self.monthly_spent + cost > monthly {
                return BudgetStatus::Exceeded;
            }
        }

        // 记录费用
        self.daily_spent += cost;
        self.monthly_spent += cost;

        // 检查是否需要警告
        if self.is_warning() {
            BudgetStatus::Warning
        } else {
            BudgetStatus::Ok
        }
    }

    pub fn is_warning(&self) -> bool {
        if !self.config.enabled {
            return false;
        }

        if let Some(daily) = self.config.daily_limit {
            if self.daily_spent / daily >= self.config.warning_threshold {
                return true;
            }
        }

        if let Some(monthly) = self.config.monthly_limit {
            if self.monthly_spent / monthly >= self.config.warning_threshold {
                return true;
            }
        }

        false
    }

    pub fn get_status(&self) -> (f64, Option<f64>, f64, Option<f64>) {
        (
            self.daily_spent,
            self.config.daily_limit,
            self.monthly_spent,
            self.config.monthly_limit,
        )
    }

    fn reset_if_needed(&mut self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        if now >= self.daily_reset {
            self.daily_spent = 0.0;
            self.daily_reset = now + 86400;
        }
        if now >= self.monthly_reset {
            self.monthly_spent = 0.0;
            self.monthly_reset = now + 2592000;
        }
    }
}
