//! 简化版 Pet Agent - TUI 示例
//!
//! ## 运行
//!
//! ```bash
//! cargo run
//! ```

mod app;
mod commands;
mod config;
mod pet;
mod ui;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

use agent_pet_rs::prelude::*;
use app::{AIResult, AppState};
use commands::Command;
use config::AppConfig;
use ui::draw_ui;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 加载配置
    let config = AppConfig::load()?;

    // 2. 检查是否有 API Key
    let has_key = config
        .ai
        .providers
        .values()
        .any(|p| p.enabled && !p.api_key.is_empty());

    if !has_key {
        println!("请先在 ~/.pet_agent/config.toml 中配置 API Key");
        println!("示例配置:");
        println!();
        println!("[ai]");
        println!("default_provider = \"openai\"");
        println!();
        println!("[ai.providers.openai]");
        println!("enabled = true");
        println!("api_key = \"sk-...\"");
        println!("model = \"gpt-4o-mini\"");
        return Ok(());
    }

    // 3. 创建 AI Manager
    let ai_config = config.to_ai_config();
    let ai = AIProviderManager::new(&ai_config)?;

    // 4. 加载或创建状态
    let mut state = AppState::load_or_default(ai, config)?;

    // 5. 添加欢迎消息
    if state.messages.is_empty() {
        state.add_system_message(&format!("Welcome! I'm {} 🐕", state.pet.name));
        state.add_system_message("Type /help for commands, or just chat!");
    }

    // 6. 初始化 TUI
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 7. 主循环
    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_secs(1); // 每秒 tick

    let result = run_app(&mut terminal, &mut state, &mut last_tick, tick_rate);

    // 8. 恢复终端
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    // 9. 处理结果
    if let Err(err) = result {
        eprintln!("Error: {}", err);
    }

    Ok(())
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    state: &mut AppState,
    last_tick: &mut Instant,
    tick_rate: Duration,
) -> Result<(), Box<dyn std::error::Error>> {
    // 创建 tokio runtime 用于执行异步任务
    let rt = tokio::runtime::Handle::current();

    while state.running {
        // 更新宠物状态 (每秒)
        if last_tick.elapsed() >= tick_rate {
            state.pet.update();
            *last_tick = Instant::now();
        }

        // 检查 AI 响应 (非阻塞)
        if let Some(rx) = &mut state.receiver {
            if let Ok(result) = rx.try_recv() {
                state.remove_last_system_message(); // 移除 "Thinking..."
                match result {
                    AIResult::Success(content) => {
                        state.add_assistant_message(&content);
                    }
                    AIResult::Error(e) => {
                        state.add_system_message(&format!("Error: {}", e));
                    }
                }
                state.pending_request = false;
                state.receiver = None;
            }
        }

        // 渲染 UI
        terminal.draw(|f| draw_ui(f, state))?;

        // 处理输入 (非阻塞)
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    handle_key(key.code, state, &rt)?;
                }
            }
        }
    }

    // 保存状态
    state.save()?;

    Ok(())
}

fn handle_key(
    key: KeyCode,
    state: &mut AppState,
    rt: &tokio::runtime::Handle,
) -> Result<(), Box<dyn std::error::Error>> {
    match key {
        KeyCode::Enter => {
            let input = state.input.trim().to_string();
            if input.is_empty() {
                return Ok(());
            }

            state.input.clear();

            if input.starts_with('/') {
                // 命令立即执行，不受 pending_request 影响
                let cmd = Command::parse(&input);
                if let Some(response) = cmd.execute(state) {
                    state.add_system_message(&response);
                }
            } else {
                // 普通消息需要检查是否有请求在进行中
                if state.pending_request {
                    state.add_system_message("Still thinking, please wait...");
                    return Ok(());
                }

                // 发送到 AI (异步)
                state.add_user_message(&input);

                // 构建消息
                let mut messages = vec![ChatMessage {
                    role: "system".to_string(),
                    content: format!(
                        "You are {}, a cute virtual pet. Be friendly, concise, and helpful. Use emojis occasionally.",
                        state.config.pet.name
                    ),
                }];
                messages.extend(state.history.to_chat_messages());

                // 创建 channel
                let (tx, rx) = mpsc::unbounded_channel();
                state.receiver = Some(rx);
                state.pending_request = true;
                state.add_system_message("Thinking...");

                // 克隆 AI manager 用于后台任务
                let ai = state.ai.clone();

                // 在后台执行 AI 调用
                rt.spawn_blocking(move || {
                    // 获取锁并执行调用
                    let mut manager = ai.lock().unwrap();
                    match manager.chat(messages) {
                        Ok(response) => {
                            let _ = tx.send(AIResult::Success(response.content));
                        }
                        Err(e) => {
                            let _ = tx.send(AIResult::Error(e.to_string()));
                        }
                    }
                });
            }
        }
        KeyCode::Char(c) => {
            state.input.push(c);
        }
        KeyCode::Backspace => {
            state.input.pop();
        }
        KeyCode::Esc => {
            state.running = false;
        }
        _ => {}
    }

    Ok(())
}
