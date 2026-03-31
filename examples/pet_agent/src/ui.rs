//! 最小化 UI 渲染模块

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::AppState;

/// 绘制 UI
pub fn draw_ui(f: &mut Frame, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // 标题栏
            Constraint::Min(5),    // 聊天区域
            Constraint::Length(3), // 输入栏
        ])
        .split(f.area());

    draw_header(f, chunks[0], state);
    draw_messages(f, chunks[1], state);
    draw_input(f, chunks[2], state);
}

/// 绘制标题栏
fn draw_header(f: &mut Frame, area: Rect, state: &AppState) {
    let energy_pct = (state.pet.energy * 100.0) as u32;
    let mood_emoji = if state.pet.mood > 0.7 {
        "😊"
    } else if state.pet.mood > 0.4 {
        "😐"
    } else {
        "😢"
    };

    let header = Line::from(vec![
        Span::styled(" Pet Agent ", Style::default().fg(Color::Cyan)),
        Span::raw(state.pet.name.clone()),
        Span::raw(" "),
        Span::raw(mood_emoji),
        Span::styled(
            format!(" Energy: {}%", energy_pct),
            Style::default().fg(if energy_pct > 30 {
                Color::Green
            } else {
                Color::Red
            }),
        ),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let paragraph = Paragraph::new(header).block(block);
    f.render_widget(paragraph, area);
}

/// 绘制消息区域
fn draw_messages(f: &mut Frame, area: Rect, state: &AppState) {
    let messages: Vec<Line> = state
        .messages
        .iter()
        .rev()
        .take(area.height as usize - 2)
        .map(|msg| {
            if msg.starts_with("> ") {
                Line::from(Span::styled(
                    msg.clone(),
                    Style::default().fg(Color::Yellow),
                ))
            } else if msg.starts_with("< ") {
                Line::from(Span::styled(msg.clone(), Style::default().fg(Color::Green)))
            } else {
                Line::from(Span::styled(
                    msg.clone(),
                    Style::default().fg(Color::DarkGray),
                ))
            }
        })
        .collect();

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let paragraph = Paragraph::new(messages)
        .block(block)
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

/// 绘制输入栏
fn draw_input(f: &mut Frame, area: Rect, state: &AppState) {
    let input_text = format!("> {}_", state.input);
    let hint = "[ESC to quit]";

    let line = Line::from(vec![
        Span::styled(hint, Style::default().fg(Color::DarkGray)),
        Span::raw(" "),
        Span::raw(input_text),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let paragraph = Paragraph::new(line).block(block);
    f.render_widget(paragraph, area);
}
