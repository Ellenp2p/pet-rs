//! 事件处理模块
//!
//! 处理键盘和鼠标事件。
//! 使用 ratatui-interact 处理输入。

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
use ratatui_interact::components::scrollable_content::handle_scrollable_content_key;

/// 处理键盘事件
pub async fn handle_key_event(key: KeyEvent, app: &mut crate::app::App) -> anyhow::Result<()> {
    // 当焦点在消息区域时，处理上下键滚动
    if app.focused_area == crate::app::FocusedArea::Messages {
        match key.code {
            KeyCode::Up => {
                app.scroll_state.scroll_up(3);
                return Ok(());
            }
            KeyCode::Down => {
                app.scroll_state.scroll_down(3, 10);
                return Ok(());
            }
            KeyCode::Home => {
                app.scroll_state.scroll_to_top();
                return Ok(());
            }
            KeyCode::End => {
                app.scroll_state.scroll_to_bottom(10);
                return Ok(());
            }
            KeyCode::PageUp => {
                app.scroll_state.page_up(10);
                return Ok(());
            }
            KeyCode::PageDown => {
                app.scroll_state.page_down(10);
                return Ok(());
            }
            _ => {}
        }
    }

    // 特殊按键处理
    match key.code {
        // 退出
        KeyCode::Esc => {
            app.should_quit = true;
            return Ok(());
        }
        // Ctrl+C 退出
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.should_quit = true;
            return Ok(());
        }
        // 回车发送消息
        KeyCode::Enter => {
            let input = app.textarea_state.text().to_string();
            if !input.is_empty() {
                // 检查是否是命令
                if input.starts_with('/') {
                    let command = crate::commands::parse_command(&input);
                    let response = crate::commands::execute_command(app, command).await;
                    app.messages
                        .push(crate::app::DisplayMessage::system(&response));
                    // 清空输入
                    app.textarea_state.clear();
                } else {
                    // 发送消息给 AI
                    app.send_message().await?;
                }
            }
            return Ok(());
        }
        // Tab 切换位置
        KeyCode::Tab => {
            app.switch_location();
            return Ok(());
        }
        // F1 喂食
        KeyCode::F(1) => {
            app.feed();
            return Ok(());
        }
        // F2 玩耍
        KeyCode::F(2) => {
            app.play();
            return Ok(());
        }
        // F3 休息
        KeyCode::F(3) => {
            app.rest();
            return Ok(());
        }
        // F4 探索
        KeyCode::F(4) => {
            app.explore();
            return Ok(());
        }
        // F5 设置
        KeyCode::F(5) => {
            app.messages
                .push(crate::app::DisplayMessage::system("设置功能待实现"));
            return Ok(());
        }
        // 数字键切换位置（仅在非编辑模式下）
        KeyCode::Char('1') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.set_location(0);
            return Ok(());
        }
        KeyCode::Char('2') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.set_location(1);
            return Ok(());
        }
        KeyCode::Char('3') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.set_location(2);
            return Ok(());
        }
        KeyCode::Char('4') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.set_location(3);
            return Ok(());
        }
        _ => {}
    }

    // 其他按键交给 TextAreaState 处理
    match key.code {
        KeyCode::Char(c) => {
            app.textarea_state.insert_char(c);
        }
        KeyCode::Backspace => {
            app.textarea_state.delete_char_backward();
        }
        KeyCode::Delete => {
            app.textarea_state.delete_char_forward();
        }
        KeyCode::Left => {
            app.textarea_state.move_left();
        }
        KeyCode::Right => {
            app.textarea_state.move_right();
        }
        KeyCode::Up => {
            app.textarea_state.move_up();
        }
        KeyCode::Down => {
            app.textarea_state.move_down();
        }
        KeyCode::Home => {
            app.textarea_state.move_line_start();
        }
        KeyCode::End => {
            app.textarea_state.move_line_end();
        }
        KeyCode::Tab => {
            app.textarea_state.insert_tab();
        }
        KeyCode::PageUp => {
            app.textarea_state.move_page_up();
        }
        KeyCode::PageDown => {
            app.textarea_state.move_page_down();
        }
        _ => {}
    }
    Ok(())
}

/// 处理鼠标事件
pub async fn handle_mouse_event(
    mouse: MouseEvent,
    app: &mut crate::app::App,
) -> anyhow::Result<()> {
    match mouse.kind {
        MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
            let (x, y) = (mouse.column, mouse.row);

            // 根据点击位置切换焦点区域
            // 标题行 (0-2行)
            if y <= 2 {
                // 忽略标题栏点击
            }
            // 内容区域 (3-12行，约)
            else if y >= 3 && y <= 12 {
                // 左边区域：宠物/位置
                if x < 40 {
                    app.focused_area = crate::app::FocusedArea::Pet;
                }
                // 右边区域：消息历史
                else {
                    app.focused_area = crate::app::FocusedArea::Messages;
                }
            }
            // 输入区域 (约 13行以后)
            else if y >= 13 {
                app.focused_area = crate::app::FocusedArea::Input;
            }

            // 检测位置标签点击 (第 3 行)
            if y == 2 {
                if x >= 2 && x <= 10 {
                    app.set_location(0); // 屋里
                } else if x >= 12 && x <= 22 {
                    app.set_location(1); // 工作室
                } else if x >= 24 && x <= 32 {
                    app.set_location(2); // 前院
                } else if x >= 34 && x <= 46 {
                    app.set_location(3); // 后院农场
                }
            }
        }
        MouseEventKind::ScrollUp => {
            // 向上滚动消息历史（当焦点在消息区域时）
            if app.focused_area == crate::app::FocusedArea::Messages {
                app.scroll_state.scroll_up(3);
            } else if app.focused_area == crate::app::FocusedArea::Input {
                // 输入框中向上滚动
                app.textarea_state.move_up();
            }
        }
        MouseEventKind::ScrollDown => {
            // 向下滚动消息历史（当焦点在消息区域时）
            if app.focused_area == crate::app::FocusedArea::Messages {
                app.scroll_state.scroll_down(3, 10);
            } else if app.focused_area == crate::app::FocusedArea::Input {
                // 输入框中向下滚动
                app.textarea_state.move_down();
            }
        }
        _ => {}
    }
    Ok(())
}
