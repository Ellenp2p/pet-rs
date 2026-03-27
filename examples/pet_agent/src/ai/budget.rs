//! 预算管理

use super::error::AIError;
use serde::{Deserialize, Serialize};

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

pub struct BudgetTracker {
    config: BudgetConfig,
    daily_spent: f64,
    monthly_spent: f64,
    daily_reset: u64,
    monthly_reset: u64,
}

impl BudgetTracker {
    pub fn new(config: BudgetConfig) -> Self {
        let now = chrono::Utc::now().timestamp() as u64;
        Self {
            config,
            daily_spent: 0.0,
            monthly_spent: 0.0,
            daily_reset: now + 86400,
            monthly_reset: now + 2592000,
        }
    }

    pub fn check_budget(&mut self, cost: f64) -> Result<(), AIError> {
        if !self.config.enabled {
            return Ok(());
        }

        self.reset_if_needed();

        if let Some(daily) = self.config.daily_limit {
            if self.daily_spent + cost > daily {
                return Err(AIError::BudgetExceeded(format!(
                    "每日预算超限: ${:.2} / ${:.2}",
                    self.daily_spent + cost,
                    daily
                )));
            }
        }

        if let Some(monthly) = self.config.monthly_limit {
            if self.monthly_spent + cost > monthly {
                return Err(AIError::BudgetExceeded(format!(
                    "每月预算超限: ${:.2} / ${:.2}",
                    self.monthly_spent + cost,
                    monthly
                )));
            }
        }

        self.daily_spent += cost;
        self.monthly_spent += cost;
        Ok(())
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
        let now = chrono::Utc::now().timestamp() as u64;
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
