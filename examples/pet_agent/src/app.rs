//! 应用状态管理模块

use crate::ai::{self, ChatMessage, ProviderManager};
use crate::animation::Animation;
use crate::config::Config;
use crate::location::Location;
use crate::memory::Memory;
use crate::pet::{Pet, PetState};

pub struct App {
    pub config: Config,
    pub pet: Pet,
    pub memory: Memory,
    pub animation: Animation,
    pub provider_manager: Option<ProviderManager>,
    pub input: String,
    pub messages: Vec<DisplayMessage>,
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

        Ok(Self {
            config,
            pet,
            memory,
            animation,
            provider_manager,
            input: String::new(),
            messages: Vec::new(),
            should_quit: false,
            is_thinking: false,
            location_index: 0,
            needs_setup,
        })
    }

    pub async fn send_message(&mut self) -> anyhow::Result<()> {
        if self.input.is_empty() {
            return Ok(());
        }

        let user_message = self.input.clone();
        self.input.clear();
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
                }
                Err(e) => {
                    self.messages.push(DisplayMessage::system(&format!("AI 错误: {}", e)));
                    self.pet.set_state(PetState::Idle);
                }
            }
        } else {
            self.messages.push(DisplayMessage::system("请先配置 AI 提供商 (/setup)"));
            self.pet.set_state(PetState::Idle);
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

    pub fn save(&self) -> anyhow::Result<()> {
        self.memory.save(&self.config.memory_path())
    }

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
}
