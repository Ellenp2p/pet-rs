//! 宠物逻辑模块

use serde::{Deserialize, Serialize};

/// 宠物状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pet {
    pub name: String,
    pub energy: f32, // 0.0 ~ 1.0
    pub mood: f32,   // 0.0 ~ 1.0
    pub last_update: u64,
}

impl Pet {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            energy: 0.8,
            mood: 0.8,
            last_update: current_timestamp(),
        }
    }

    /// 每秒更新状态
    pub fn update(&mut self) {
        let now = current_timestamp();
        let elapsed = now - self.last_update;
        if elapsed == 0 {
            return;
        }

        // 能量缓慢下降 (每秒 -0.001)
        self.energy = (self.energy - elapsed as f32 * 0.001).max(0.0);

        // 心情与能量相关
        self.mood = (self.energy * 0.7 + 0.3).min(1.0);

        self.last_update = now;
    }

    /// 喂食
    pub fn feed(&mut self) -> String {
        self.update();
        let gain = 0.2;
        self.energy = (self.energy + gain).min(1.0);
        self.mood = (self.mood + 0.1).min(1.0);
        format!(
            "*nom nom* Thanks for the food! (+{:.0}% energy)",
            gain * 100.0
        )
    }

    /// 玩耍
    pub fn play(&mut self) -> String {
        self.update();
        let energy_cost = 0.05;
        let mood_gain = 0.15;
        if self.energy < energy_cost {
            return "Too tired to play... Need some food first!".to_string();
        }
        self.energy = (self.energy - energy_cost).max(0.0);
        self.mood = (self.mood + mood_gain).min(1.0);
        format!(
            "Wheee! That was fun! (+{:.0}% mood, -{:.0}% energy)",
            mood_gain * 100.0,
            energy_cost * 100.0
        )
    }

    /// 休息
    pub fn rest(&mut self) -> String {
        self.update();
        let gain = 0.3;
        self.energy = (self.energy + gain).min(1.0);
        format!("*yawn* Feeling refreshed! (+{:.0}% energy)", gain * 100.0)
    }

    /// 获取状态描述
    pub fn status_description(&self) -> String {
        let energy_bar = bar(self.energy);
        let mood_bar = bar(self.mood);
        let mood_emoji = if self.mood > 0.7 {
            "😊"
        } else if self.mood > 0.4 {
            "😐"
        } else {
            "😢"
        };

        format!(
            "{} {}\n  Energy: {}\n  Mood:   {}",
            self.name, mood_emoji, energy_bar, mood_bar
        )
    }
}

fn bar(value: f32) -> String {
    let filled = (value * 10.0) as usize;
    let empty = 10 - filled;
    format!(
        "[{}{}] {:.0}%",
        "█".repeat(filled),
        "░".repeat(empty),
        value * 100.0
    )
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
