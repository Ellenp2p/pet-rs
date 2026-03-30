//! 速率限制器

use super::provider::RateLimitConfig;
use super::AIError;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// 速率限制器
pub struct RateLimiter {
    config: RateLimitConfig,
    request_history: VecDeque<Instant>,
    token_history: VecDeque<(Instant, u32)>,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            request_history: VecDeque::new(),
            token_history: VecDeque::new(),
        }
    }

    pub fn check_request(&mut self, tokens: u32) -> Result<(), AIError> {
        if !self.config.enabled {
            return Ok(());
        }

        let now = Instant::now();
        self.cleanup_history(now);

        // 检查每分钟请求限制
        if let Some(rpm) = self.config.requests_per_minute {
            let minute_ago = now - Duration::from_secs(60);
            let recent = self
                .request_history
                .iter()
                .filter(|&&t| t > minute_ago)
                .count();
            if recent >= rpm as usize {
                return Err(AIError::RateLimited(format!(
                    "每分钟请求限制已达 {} 次",
                    rpm
                )));
            }
        }

        // 检查每小时请求限制
        if let Some(rph) = self.config.requests_per_hour {
            let hour_ago = now - Duration::from_secs(3600);
            let recent = self
                .request_history
                .iter()
                .filter(|&&t| t > hour_ago)
                .count();
            if recent >= rph as usize {
                return Err(AIError::RateLimited(format!(
                    "每小时请求限制已达 {} 次",
                    rph
                )));
            }
        }

        // 检查每分钟 Token 限制
        if let Some(tpm) = self.config.tokens_per_minute {
            let minute_ago = now - Duration::from_secs(60);
            let recent_tokens: u32 = self
                .token_history
                .iter()
                .filter(|(t, _)| *t > minute_ago)
                .map(|(_, tokens)| tokens)
                .sum();
            if recent_tokens + tokens > tpm {
                return Err(AIError::RateLimited(format!(
                    "每分钟 Token 限制已达 {}",
                    tpm
                )));
            }
        }

        // 检查每小时 Token 限制
        if let Some(tph) = self.config.tokens_per_hour {
            let hour_ago = now - Duration::from_secs(3600);
            let recent_tokens: u32 = self
                .token_history
                .iter()
                .filter(|(t, _)| *t > hour_ago)
                .map(|(_, tokens)| tokens)
                .sum();
            if recent_tokens + tokens > tph {
                return Err(AIError::RateLimited(format!(
                    "每小时 Token 限制已达 {}",
                    tph
                )));
            }
        }

        self.request_history.push_back(now);
        self.token_history.push_back((now, tokens));
        Ok(())
    }

    pub fn wait_if_needed(&mut self, tokens: u32) -> Result<(), AIError> {
        if !self.config.enabled {
            return Ok(());
        }

        loop {
            match self.check_request(tokens) {
                Ok(()) => return Ok(()),
                Err(AIError::RateLimited(msg)) => {
                    let wait = self.calculate_wait_time();
                    log::warn!("速率限制: {}，等待 {:?}", msg, wait);
                    std::thread::sleep(wait);
                }
                Err(e) => return Err(e),
            }
        }
    }

    fn calculate_wait_time(&self) -> Duration {
        let now = Instant::now();
        let minute_ago = now - Duration::from_secs(60);

        if let Some(oldest) = self.request_history.front() {
            if *oldest > minute_ago {
                let wait_until = *oldest + Duration::from_secs(60);
                return wait_until.duration_since(now) + Duration::from_millis(100);
            }
        }
        Duration::from_secs(1)
    }

    fn cleanup_history(&mut self, now: Instant) {
        let hour_ago = now - Duration::from_secs(3600);
        while let Some(front) = self.request_history.front() {
            if *front < hour_ago {
                self.request_history.pop_front();
            } else {
                break;
            }
        }
        while let Some((front, _)) = self.token_history.front() {
            if *front < hour_ago {
                self.token_history.pop_front();
            } else {
                break;
            }
        }
    }

    pub fn get_status(&self) -> (usize, usize, u32, u32) {
        let now = Instant::now();
        let minute_ago = now - Duration::from_secs(60);
        let hour_ago = now - Duration::from_secs(3600);

        let req_min = self
            .request_history
            .iter()
            .filter(|&&t| t > minute_ago)
            .count();
        let req_hour = self
            .request_history
            .iter()
            .filter(|&&t| t > hour_ago)
            .count();
        let tok_min: u32 = self
            .token_history
            .iter()
            .filter(|(t, _)| *t > minute_ago)
            .map(|(_, t)| t)
            .sum();
        let tok_hour: u32 = self
            .token_history
            .iter()
            .filter(|(t, _)| *t > hour_ago)
            .map(|(_, t)| t)
            .sum();

        (req_min, req_hour, tok_min, tok_hour)
    }
}
