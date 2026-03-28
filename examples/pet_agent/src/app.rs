//! 应用状态管理模块

use crate::ai::{self, ChatMessage, ProviderManager};
use crate::animation::Animation;
use crate::config::Config;
use crate::location::Location;
use crate::memory::Memory;
use crate::pet::{Pet, PetState};
use tui_textarea::TextArea;
use std::time::Instant;
use std::time::Duration;

/// Toast 通知类型
#[derive(Debug, Clone, PartialEq)]
pub enum ToastType {
    Info,
    Success,
    Warning,
    Error,
}

/// Toast 通知
#[derive(Debug, Clone)]
pub struct Toast {
    pub message: String,
    pub toast_type: ToastType,
    pub created_at: Instant,
    pub duration: Duration,
}

pub struct App {
    pub config: Config,
    pub pet: Pet,
    pub memory: Memory,
    pub animation: Animation,
    pub provider_manager: Option<ProviderManager>,
    pub textarea: TextArea<'static>,  // 使用 tui-textarea
    pub character_index: usize,  // 保留用于兼容性
    pub messages: Vec<DisplayMessage>,
    pub toasts: Vec<Toast>,  // Toast 通知
    pub should_quit: bool,
    pub is_thinking: bool,
    pub location_index: usize,
    pub needs_setup: bool,
}

pub struct DisplayMessage {
    pub sender: String,
    pub content: String,
    pub is_system: bool,
}

impl DisplayMessage {
    pub fn user(content: &str) -> Self {
        Self { sender: "你".to_string(), content: content.to_string(), is_system: false }
    }
    pub fn pet(name: &str, content: &str) -> Self {
        Self { sender: name.to_string(), content: content.to_string(), is_system: false }
    }
    pub fn system(content: &str) -> Self {
        Self { sender: "系统".to_string(), content: content.to_string(), is_system: true }
    }
}

impl App {
    pub fn new() -> anyhow::Result<Self> {
        let config = Config::load()?;
        let memory = Memory::load(&config.memory_path())?;
        let animation = Animation::new(config.settings.animation_speed);
        let mut pet = Pet::new("Buddy");
        pet.size = config.settings.dog_size;

        let needs_setup = config.needs_setup();

        let provider_manager = if !needs_setup {
            let ai_config = ai::manager::AIConfig {
                auto_switch: config.ai.auto_switch,
                switch_order: config.ai.switch_order.clone(),
                providers: config.ai.providers.clone(),
            };
            ProviderManager::new(&ai_config, &config.usage_path(), config.ai.budget.clone()).ok()
        } else {
            None
        };

        // 创建 tui-textarea 实例
        let mut textarea = TextArea::default();
        textarea.set_block(
            ratatui::widgets::Block::default()
                .borders(ratatui::widgets::Borders::ALL)
                .title("输入消息")
        );

        Ok(Self {
            config,
            pet,
            memory,
            animation,
            provider_manager,
            textarea,
            character_index: 0,
            messages: Vec::new(),
            toasts: Vec::new(),
            should_quit: false,
            is_thinking: false,
            location_index: 0,
            needs_setup,
        })
    }

    pub async fn send_message(&mut self) -> anyhow::Result<()> {
        // 从 textarea 获取输入内容
        let input = self.textarea.lines().join("\n");
        if input.is_empty() {
            return Ok(());
        }

        // 清空 textarea
        self.textarea = TextArea::default();
        self.textarea.set_block(
            ratatui::widgets::Block::default()
                .borders(ratatui::widgets::Borders::ALL)
                .title("输入消息")
        );

        let user_message = input;
        self.messages.push(DisplayMessage::user(&user_message));
        self.pet.set_state(PetState::Thinking);
        self.is_thinking = true;

        let system_prompt = ai::create_system_prompt(self.pet.location.name(), self.pet.state.name(), &self.pet.name);
        let history = self.memory.get_recent_context(10);
        let messages = ai::create_messages(&system_prompt, history, &user_message);

        if let Some(ref mut manager) = self.provider_manager {
            match manager.chat_with_fallback(messages).await {
                Ok(response) => {
                    self.messages.push(DisplayMessage::pet(&self.pet.name, &response));
                    self.memory.add_conversation(self.pet.location.name(), &user_message, &response);
                    self.pet.set_state(PetState::Happy);
                    self.pet.boost_happiness(5.0);
                    self.messages.push(DisplayMessage::system(&format!("📊 {}", manager.usage_summary())));
                    self.add_toast("消息发送成功", ToastType::Success);
                }
                Err(e) => {
                    self.messages.push(DisplayMessage::system(&format!("AI 错误: {}", e)));
                    self.pet.set_state(PetState::Idle);
                    self.add_toast(&format!("错误: {}", e), ToastType::Error);
                }
            }
        } else {
            self.messages.push(DisplayMessage::system("请先配置 AI 提供商 (/setup)"));
            self.pet.set_state(PetState::Idle);
            self.add_toast("请先配置 AI 提供商", ToastType::Warning);
        }

        self.is_thinking = false;
        Ok(())
    }

    pub fn switch_location(&mut self) {
        self.location_index = (self.location_index + 1) % Location::all().len();
        let loc = Location::from_index(self.location_index);
        self.pet.move_to(loc);
        self.messages.push(DisplayMessage::system(&format!("{} 移动到了 {} {}", self.pet.name, loc.emoji(), loc.name())));
    }

    pub fn set_location(&mut self, index: usize) {
        self.location_index = index;
        let loc = Location::from_index(index);
        self.pet.move_to(loc);
        self.messages.push(DisplayMessage::system(&format!("{} 移动到了 {} {}", self.pet.name, loc.emoji(), loc.name())));
    }

    pub fn feed(&mut self) {
        self.pet.restore_energy(20.0);
        self.pet.boost_happiness(10.0);
        self.pet.set_state(PetState::Happy);
        self.messages.push(DisplayMessage::system(&format!("你喂了 {}，它很开心！🍖", self.pet.name)));
    }

    pub fn play(&mut self) {
        self.pet.boost_happiness(20.0);
        self.pet.restore_energy(-10.0);
        self.pet.set_state(PetState::Playing);
        self.messages.push(DisplayMessage::system(&format!("你和 {} 玩耍了！🎾", self.pet.name)));
    }

    pub fn rest(&mut self) {
        self.pet.restore_energy(30.0);
        self.pet.set_state(PetState::Sleeping);
        self.messages.push(DisplayMessage::system(&format!("{} 正在休息...💤", self.pet.name)));
    }

    pub fn explore(&mut self) {
        self.pet.restore_energy(-15.0);
        self.pet.boost_happiness(15.0);
        self.pet.set_state(PetState::Working);
        self.messages.push(DisplayMessage::system(&format!("{} 开始探索周围...🔍", self.pet.name)));
    }

    pub fn clear_messages(&mut self) {
        self.messages.clear();
    }

    // ========== Toast 通知 ==========

    /// 添加 Toast 通知
    pub fn add_toast(&mut self, message: &str, toast_type: ToastType) {
        // 截断过长的消息
        let mut msg = message.to_string();
        if msg.len() > 60 {
            msg.truncate(57);
            msg.push_str("...");
        }
        
        // 移除换行符，保持一行
        msg = msg.replace('\n', " ").replace('\r', "");
        
        // 限制 toast 数量
        if self.toasts.len() >= 5 {
            self.toasts.remove(0);
        }
        
        self.toasts.push(Toast {
            message: msg,
            toast_type,
            created_at: Instant::now(),
            duration: Duration::from_secs(3),
        });
    }

    /// 更新 Toast（移除过期的通知）
    pub fn update_toasts(&mut self) {
        let now = Instant::now();
        self.toasts.retain(|t| now.duration_since(t.created_at) < t.duration);
    }

    // ========== 其他方法 ==========

    pub fn provider_status(&self) -> String {
        if let Some(ref pm) = self.provider_manager {
            format!("{} | {}", pm.current_provider_name(), pm.rate_limit_status())
        } else {
            "未配置".to_string()
        }
    }

    pub fn usage_stats(&self) -> String {
        if let Some(ref pm) = self.provider_manager {
            pm.usage_summary()
        } else {
            "无数据".to_string()
        }
    }

    pub fn export_usage(&self, format: &str) -> Result<String, String> {
        if let Some(ref pm) = self.provider_manager {
            match format {
                "json" => pm.export_usage_json().map_err(|e| format!("{}", e)),
                "csv" => Ok(pm.export_usage_csv()),
                _ => Err("不支持的格式".to_string()),
            }
        } else {
            Err("未配置".to_string())
        }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        self.memory.save(&self.config.memory_path())
    }
}
