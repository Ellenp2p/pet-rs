//! 桌面宠物 Agent - TUI 示例
//!
//! 一个基于 TUI 的桌面宠物，可以与 AI 对话、执行任务、记住偏好。
//!
//! ## 运行
//!
//! ```bash
//! cargo run --bin pet-agent
//! ```

mod ai;
mod animation;
mod app;
mod commands;
mod config;
mod event;
mod location;
mod memory;
mod pet;
mod ui;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化应用
    let mut app = app::App::new()?;

    // 如果需要设置，先进行设置
    if app.needs_setup {
        println!("欢迎使用桌面宠物 Agent！");
        println!("请输入你的 OpenRouter API Key:");
        println!("(可以在 https://openrouter.ai/keys 获取)");
        print!("> ");
        io::Write::flush(&mut io::stdout())?;

        let mut api_key = String::new();
        io::stdin().read_line(&mut api_key)?;
        let api_key = api_key.trim();

        if api_key.is_empty() {
            println!("API Key 不能为空！");
            return Ok(());
        }

        // 创建提供商配置
        let provider_config = crate::ai::provider::ProviderConfig::new(
            crate::ai::provider::ProviderType::OpenRouter,
            api_key,
        );
        app.config.ai.providers.push(provider_config);
        app.config.save()?;
        println!("配置已保存！启动宠物...\n");
    }

    // 设置终端
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 运行应用
    let result = run_app(&mut terminal, &mut app).await;

    // 恢复终端
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    // 保存状态
    if let Err(e) = app.save() {
        eprintln!("保存状态失败: {}", e);
    }

    // 处理错误
    if let Err(err) = result {
        eprintln!("应用错误: {:?}", err);
    }

    Ok(())
}

/// 运行应用
async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut app::App,
) -> anyhow::Result<()> {
    let event_handler = event::EventHandler::new(app.config.settings.animation_speed);

    // 添加欢迎消息
    app.messages.push(app::DisplayMessage::system(&format!(
        "欢迎！我是 {}，你的智能宠物助手！🐕",
        app.pet.name
    )));
    app.messages.push(app::DisplayMessage::system(
        "输入消息和我聊天，或者按 /help 查看帮助",
    ));

    loop {
        // 渲染 UI
        terminal.draw(|f| ui::render(f, app))?;

        // 处理事件
        match event_handler.next()? {
            event::AppEvent::Key(key) => {
                event::handle_key_event(key, app).await?;
            }
            event::AppEvent::Mouse(mouse) => {
                event::handle_mouse_event(mouse, app).await?;
            }
            event::AppEvent::None => {
                // 无事件，继续
            }
        }

        // 检查是否退出
        if app.should_quit {
            break;
        }
    }

    Ok(())
}
