//! TUI 终端管理模块
//!
//! 使用 EventStream 和 tokio::select! 实现高效的事件处理

use std::{
    io::Stderr,
    ops::{Deref, DerefMut},
    time::Duration,
};

use anyhow::Result;
use futures::{FutureExt, StreamExt};
use ratatui::{backend::CrosstermBackend as Backend, Terminal};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

use crossterm::{
    cursor,
    event::{Event as CrosstermEvent, EventStream, KeyEvent, KeyEventKind, MouseEvent},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};

/// 应用事件
#[derive(Debug, Clone)]
pub enum Event {
    /// 初始化
    Init,
    /// 退出
    Quit,
    /// 错误
    Error,
    /// 定时更新
    Tick,
    /// 渲染
    Render,
    /// 键盘事件
    Key(KeyEvent),
    /// 鼠标事件
    Mouse(MouseEvent),
    /// 窗口大小变化
    Resize(u16, u16),
    /// AI 响应成功
    AiResponse(String),
    /// AI 错误
    AiError(String),
    /// Toast 通知 (message, is_error)
    Toast(String, bool),
}

/// TUI 终端管理器
pub struct Tui {
    pub terminal: Terminal<Backend<Stderr>>,
    pub task: tokio::task::JoinHandle<()>,
    pub cancellation_token: tokio_util::sync::CancellationToken,
    pub event_rx: UnboundedReceiver<Event>,
    pub event_tx: UnboundedSender<Event>,
    pub frame_rate: f64,
    pub tick_rate: f64,
    pub mouse: bool,
}

impl Tui {
    /// 创建新的 TUI
    pub fn new() -> Result<Self> {
        let tick_rate = 4.0; // 4 ticks per second
        let frame_rate = 30.0; // 30 frames per second
        let terminal = Terminal::new(Backend::new(std::io::stderr()))?;
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let cancellation_token = tokio_util::sync::CancellationToken::new();
        let task = tokio::spawn(async {});
        let mouse = true; // 默认启用鼠标

        Ok(Self {
            terminal,
            task,
            cancellation_token,
            event_rx,
            event_tx,
            frame_rate,
            tick_rate,
            mouse,
        })
    }

    /// 设置 tick rate
    pub fn tick_rate(mut self, tick_rate: f64) -> Self {
        self.tick_rate = tick_rate;
        self
    }

    /// 设置 frame rate
    pub fn frame_rate(mut self, frame_rate: f64) -> Self {
        self.frame_rate = frame_rate;
        self
    }

    /// 设置鼠标支持
    pub fn mouse(mut self, mouse: bool) -> Self {
        self.mouse = mouse;
        self
    }

    /// 启动事件处理
    pub fn start(&mut self) {
        let tick_delay = Duration::from_secs_f64(1.0 / self.tick_rate);
        let render_delay = Duration::from_secs_f64(1.0 / self.frame_rate);
        self.cancel();
        self.cancellation_token = tokio_util::sync::CancellationToken::new();
        let _cancellation_token = self.cancellation_token.clone();
        let _event_tx = self.event_tx.clone();

        self.task = tokio::spawn(async move {
            let mut reader = EventStream::new();
            let mut tick_interval = tokio::time::interval(tick_delay);
            let mut render_interval = tokio::time::interval(render_delay);

            _event_tx.send(Event::Init).unwrap();

            loop {
                let tick_delay = tick_interval.tick();
                let render_delay = render_interval.tick();
                let crossterm_event = reader.next().fuse();

                tokio::select! {
                    _ = _cancellation_token.cancelled() => {
                        break;
                    }
                    maybe_event = crossterm_event => {
                        match maybe_event {
                            Some(Ok(evt)) => {
                                match evt {
                                    CrosstermEvent::Key(key) => {
                                        if key.kind == KeyEventKind::Press {
                                            _event_tx.send(Event::Key(key)).unwrap();
                                            // 立即触发渲染
                                            _event_tx.send(Event::Render).unwrap();
                                        }
                                    }
                                    CrosstermEvent::Mouse(mouse) => {
                                        _event_tx.send(Event::Mouse(mouse)).unwrap();
                                    }
                                    CrosstermEvent::Resize(x, y) => {
                                        _event_tx.send(Event::Resize(x, y)).unwrap();
                                    }
                                    _ => {}
                                }
                            }
                            Some(Err(_)) => {
                                _event_tx.send(Event::Error).unwrap();
                            }
                            None => {}
                        }
                    }
                    _ = tick_delay => {
                        _event_tx.send(Event::Tick).unwrap();
                    }
                    _ = render_delay => {
                        _event_tx.send(Event::Render).unwrap();
                    }
                }
            }
        });
    }

    /// 停止事件处理
    pub fn stop(&self) -> Result<()> {
        self.cancel();
        let mut counter = 0;
        while !self.task.is_finished() {
            std::thread::sleep(Duration::from_millis(1));
            counter += 1;
            if counter > 50 {
                self.task.abort();
            }
            if counter > 100 {
                log::error!("Failed to abort task in 100 milliseconds");
                break;
            }
        }
        Ok(())
    }

    /// 进入终端模式
    pub fn enter(&mut self) -> Result<()> {
        crossterm::terminal::enable_raw_mode()?;
        crossterm::execute!(std::io::stderr(), EnterAlternateScreen, cursor::Hide)?;
        if self.mouse {
            crossterm::execute!(
                std::io::stderr(),
                crossterm::event::EnableMouseCapture
            )?;
        }
        self.start();
        Ok(())
    }

    /// 退出终端模式
    pub fn exit(&mut self) -> Result<()> {
        self.stop()?;
        if crossterm::terminal::is_raw_mode_enabled()? {
            self.flush()?;
            if self.mouse {
                crossterm::execute!(
                    std::io::stderr(),
                    crossterm::event::DisableMouseCapture
                )?;
            }
            crossterm::execute!(std::io::stderr(), LeaveAlternateScreen, cursor::Show)?;
            crossterm::terminal::disable_raw_mode()?;
        }
        Ok(())
    }

    /// 取消任务
    pub fn cancel(&self) {
        self.cancellation_token.cancel();
    }

    /// 获取下一个事件
    pub async fn next(&mut self) -> Option<Event> {
        self.event_rx.recv().await
    }
}

impl Deref for Tui {
    type Target = Terminal<Backend<Stderr>>;

    fn deref(&self) -> &Self::Target {
        &self.terminal
    }
}

impl DerefMut for Tui {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.terminal
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        self.exit().unwrap();
    }
}
