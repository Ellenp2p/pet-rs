//! UI 渲染模块
//!
//! 使用 ratatui 渲染界面。

use ratatui::{
    layout::{Constraint, Direction, Layout, Position},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs, Wrap},
    Frame,
};

use crate::app::App;
use crate::location::Location;

/// 渲染 UI
pub fn render(f: &mut Frame, app: &App) {
    // UI 渲染逻辑

    // 主布局：标题、内容、Toast、输入
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // 标题
            Constraint::Min(10),   // 内容
            Constraint::Length(6), // Toast 区域
            Constraint::Length(5), // 输入区域
        ])
        .split(f.area());

    // 渲染标题
    render_title(f, main_chunks[0]);

    // 内容区域：左侧小狗、右侧对话历史
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40), // 小狗区域
            Constraint::Percentage(60), // 对话历史
        ])
        .split(main_chunks[1]);

    // 渲染小狗
    render_pet(f, content_chunks[0], app);

    // 渲染对话历史
    render_messages(f, content_chunks[1], app);

    // 渲染 Toast 通知
    render_toasts(f, main_chunks[2], app);

    // 渲染输入区域（使用 tui-textarea）
    render_input(f, main_chunks[3], app);
}

/// 渲染标题
fn render_title(f: &mut Frame, area: ratatui::layout::Rect) {
    let title = Paragraph::new(Line::from(vec![Span::styled(
        "🐕 Buddy - 智能宠物助手",
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )]))
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, area);
}

/// 渲染小狗区域
fn render_pet(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // 位置标签
            Constraint::Min(5),    // 小狗显示
            Constraint::Length(4), // 提供商信息
        ])
        .split(area);

    // 位置标签
    let locations = Location::all();
    let titles: Vec<String> = locations
        .iter()
        .map(|l| format!("{} {}", l.emoji(), l.name()))
        .collect();

    let tabs = Tabs::new(titles)
        .block(Block::default().title("位置").borders(Borders::ALL))
        .select(app.location_index)
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(tabs, chunks[0]);

    // 小狗显示
    let pet_art = app.pet.ascii_art();
    let pet_text: Vec<Line> = pet_art
        .iter()
        .map(|line| Line::from(line.as_str()))
        .collect();

    let pet_paragraph = Paragraph::new(pet_text)
        .block(
            Block::default()
                .title(format!(
                    "{} {}",
                    app.pet.location.emoji(),
                    app.pet.location.name()
                ))
                .borders(Borders::ALL),
        )
        .style(Style::default().fg(Color::White));

    f.render_widget(pet_paragraph, chunks[1]);

    // 提供商信息
    let provider_info = vec![
        Line::from(vec![
            Span::styled("🤖 ", Style::default().fg(Color::White)),
            Span::styled(app.provider_status(), Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled("📊 ", Style::default().fg(Color::White)),
            Span::styled(app.usage_stats(), Style::default().fg(Color::Green)),
        ]),
    ];

    let provider_paragraph =
        Paragraph::new(provider_info).block(Block::default().title("AI").borders(Borders::ALL));

    f.render_widget(provider_paragraph, chunks[2]);
}

/// 渲染对话历史
fn render_messages(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let messages: Vec<ListItem> = app
        .messages
        .iter()
        .rev()
        .take(area.height as usize - 2) // 减去边框
        .map(|msg| {
            let style = if msg.is_system {
                Style::default().fg(Color::Gray)
            } else if msg.sender == "你" {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Cyan)
            };

            let content = format!("[{}]: {}", msg.sender, msg.content);
            ListItem::new(content).style(style)
        })
        .collect();

    let messages_list = List::new(messages)
        .block(Block::default().title("对话历史").borders(Borders::ALL))
        .style(Style::default().fg(Color::White));

    f.render_widget(messages_list, area);
}

/// 渲染输入区域（使用 tui-textarea）
fn render_input(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // 输入框
            Constraint::Length(2), // 快捷键提示
        ])
        .split(area);

    // 渲染 tui-textarea
    f.render_widget(&app.textarea, chunks[0]);

    // 快捷键提示
    let shortcuts = Line::from(vec![
        Span::styled("[F1] 喂食  ", Style::default().fg(Color::Green)),
        Span::styled("[F2] 玩耍  ", Style::default().fg(Color::Green)),
        Span::styled("[F3] 休息  ", Style::default().fg(Color::Green)),
        Span::styled("[F4] 探索  ", Style::default().fg(Color::Green)),
        Span::styled("[Tab] 切换位置  ", Style::default().fg(Color::Yellow)),
        Span::styled("[Esc] 退出", Style::default().fg(Color::Red)),
    ]);

    let shortcuts_paragraph = Paragraph::new(shortcuts)
        .block(Block::default().borders(Borders::NONE))
        .wrap(Wrap { trim: true });

    f.render_widget(shortcuts_paragraph, chunks[1]);
}

/// 渲染 Toast 通知
fn render_toasts(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let toast_items: Vec<ListItem> = app
        .toasts
        .iter()
        .take(5) // 最多显示 5 条
        .map(|toast| {
            let style = match toast.toast_type {
                crate::app::ToastType::Info => Style::default().fg(Color::Cyan),
                crate::app::ToastType::Success => Style::default().fg(Color::Green),
                crate::app::ToastType::Warning => Style::default().fg(Color::Yellow),
                crate::app::ToastType::Error => Style::default().fg(Color::Red),
            };

            // 截断消息以适应宽度
            let max_width = area.width.saturating_sub(4) as usize;
            let mut msg = toast.message.clone();
            if msg.len() > max_width {
                msg.truncate(max_width.saturating_sub(3));
                msg.push_str("...");
            }

            ListItem::new(Line::from(Span::styled(msg, style)))
        })
        .collect();

    let toast_list = List::new(toast_items).block(
        Block::default()
            .borders(Borders::ALL)
            .title("通知")
            .border_style(Style::default().fg(Color::Yellow)),
    );

    f.render_widget(toast_list, area);
}
