//! 命令处理模块

use crate::app::App;

pub enum Command {
    Help,
    Status,
    Stats,
    Location,
    Memory,
    Clear,
    Quit,
    Feed,
    Play,
    Rest,
    Explore,
    ProviderList,
    ProviderSwitch(String),
    ProviderTest,
    ModelList,
    ModelSwitch(String),
    Export(String),
    Unknown(String),
}

pub fn parse_command(input: &str) -> Command {
    let input = input.trim();
    if !input.starts_with('/') {
        return Command::Unknown(input.to_string());
    }

    let parts: Vec<&str> = input[1..].splitn(2, ' ').collect();
    let cmd = parts[0].to_lowercase();
    let args = if parts.len() > 1 { parts[1].trim() } else { "" };

    match cmd.as_str() {
        "help" | "h" | "?" => Command::Help,
        "status" | "s" => Command::Status,
        "stats" => Command::Stats,
        "location" | "loc" | "l" => Command::Location,
        "memory" | "mem" | "m" => Command::Memory,
        "clear" | "c" => Command::Clear,
        "quit" | "q" | "exit" => Command::Quit,
        "feed" | "f" => Command::Feed,
        "play" | "p" => Command::Play,
        "rest" | "r" => Command::Rest,
        "explore" | "e" => Command::Explore,
        "provider" => {
            let sub: Vec<&str> = args.splitn(2, ' ').collect();
            match sub[0].to_lowercase().as_str() {
                "list" => Command::ProviderList,
                "switch" if sub.len() > 1 => Command::ProviderSwitch(sub[1].to_string()),
                "test" => Command::ProviderTest,
                _ => Command::ProviderList,
            }
        }
        "model" => {
            let sub: Vec<&str> = args.splitn(2, ' ').collect();
            match sub[0].to_lowercase().as_str() {
                "list" => Command::ModelList,
                "switch" if sub.len() > 1 => Command::ModelSwitch(sub[1].to_string()),
                _ => Command::ModelList,
            }
        }
        "export" => Command::Export(args.to_string()),
        _ => Command::Unknown(input.to_string()),
    }
}

pub async fn execute_command(app: &mut App, command: Command) -> String {
    match command {
        Command::Help => r#"
命令:
  /help              显示帮助
  /status            显示状态
  /stats             显示费用统计
  /location          切换位置
  /memory            查看记忆
  /clear             清空消息
  /quit              退出
  /feed              喂食
  /play              玩耍
  /rest              休息
  /explore           探索

提供商:
  /provider list     显示提供商
  /provider switch X 切换提供商
  /provider test     测试连接

模型:
  /model list        显示模型
  /model switch X    切换模型

导出:
  /export json       导出 JSON
  /export csv        导出 CSV

快捷键:
  F1-F4  动作
  Tab    切换位置
  Esc    退出
"#.to_string(),

        Command::Status => {
            let provider = app.provider_status();
            format!(
                "📍 {} {} | ⚡ {:.0}% | 😊 {:.0}%\n🤖 {}\n🧠 {}",
                app.pet.location.emoji(), app.pet.location.name(),
                app.pet.energy, app.pet.happiness,
                provider, app.memory.summary()
            )
        }

        Command::Stats => {
            format!("💰 费用统计\n{}", app.usage_stats())
        }

        Command::Location => {
            app.switch_location();
            format!("切换到: {} {}", app.pet.location.emoji(), app.pet.location.name())
        }

        Command::Memory => {
            format!("记忆:\n  对话: {}\n  偏好: {}\n  知识: {}",
                app.memory.conversations.len(),
                app.memory.preferences.len(),
                app.memory.knowledge.len()
            )
        }

        Command::Clear => {
            app.clear_messages();
            "消息已清空".to_string()
        }

        Command::Quit => {
            app.should_quit = true;
            "再见！".to_string()
        }

        Command::Feed => {
            app.feed();
            format!("喂了 {}", app.pet.name)
        }

        Command::Play => {
            app.play();
            format!("和 {} 玩耍", app.pet.name)
        }

        Command::Rest => {
            app.rest();
            format!("{} 在休息", app.pet.name)
        }

        Command::Explore => {
            app.explore();
            format!("{} 在探索", app.pet.name)
        }

        Command::ProviderList => {
            let names: Vec<&str> = app.config.ai.switch_order.iter().map(|s| s.as_str()).collect();
            let current = app.provider_status();
            format!("提供商:\n  当前: {}\n  顺序: {}", current, names.join(" → "))
        }

        Command::ProviderSwitch(name) => {
            if let Some(ref mut pm) = app.provider_manager {
                match pm.switch_provider(&name) {
                    Ok(_) => format!("已切换到 {}", name),
                    Err(e) => format!("切换失败: {}", e),
                }
            } else {
                "未配置提供商".to_string()
            }
        }

        Command::ProviderTest => {
            "测试连接... (功能待实现)".to_string()
        }

        Command::ModelList => {
            let current = app.provider_status();
            format!("当前模型: {}\n使用 /model switch <model> 切换", current)
        }

        Command::ModelSwitch(_model) => {
            "切换模型功能待实现".to_string()
        }

        Command::Export(format) => {
            match app.export_usage(&format) {
                Ok(data) => {
                    let path = dirs::home_dir().unwrap_or_default()
                        .join(".pet_agent")
                        .join(format!("usage_export.{}", format));
                    if std::fs::write(&path, &data).is_ok() {
                        format!("已导出到 {:?}", path)
                    } else {
                        "导出失败".to_string()
                    }
                }
                Err(e) => format!("导出错误: {}", e),
            }
        }

        Command::Unknown(cmd) => {
            format!("未知命令: {} (输入 /help)", cmd)
        }
    }
}
