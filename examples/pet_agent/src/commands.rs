//! 简化命令处理模块

use crate::app::AppState;

pub enum Command {
    Help,
    Status,
    Feed,
    Play,
    Rest,
    Quit,
    Clear,
    Unknown(String),
}

impl Command {
    pub fn parse(input: &str) -> Self {
        match input.trim().to_lowercase().as_str() {
            "/help" | "/h" | "?" => Command::Help,
            "/status" | "/s" => Command::Status,
            "/feed" | "/f" => Command::Feed,
            "/play" | "/p" => Command::Play,
            "/rest" | "/r" => Command::Rest,
            "/quit" | "/q" | "exit" => Command::Quit,
            "/clear" | "/c" => Command::Clear,
            other => Command::Unknown(other.to_string()),
        }
    }

    pub fn execute(self, state: &mut AppState) -> Option<String> {
        match self {
            Command::Help => Some(help_text()),
            Command::Status => Some(state.pet.status_description()),
            Command::Feed => Some(state.pet.feed()),
            Command::Play => Some(state.pet.play()),
            Command::Rest => Some(state.pet.rest()),
            Command::Quit => {
                state.running = false;
                None
            }
            Command::Clear => {
                state.history.clear();
                state.messages.clear();
                Some("Chat history cleared.".to_string())
            }
            Command::Unknown(cmd) => {
                Some(format!("Unknown command: {}. Type /help for help.", cmd))
            }
        }
    }
}

fn help_text() -> String {
    r#"Pet Agent Commands:
  /help   - Show this help
  /status - Show pet status
  /feed   - Feed your pet (+20% energy)
  /play   - Play with your pet (+15% mood, -5% energy)
  /rest   - Let your pet rest (+30% energy)
  /clear  - Clear chat history
  /quit   - Save and exit

Just type to chat with your pet!"#
        .to_string()
}
