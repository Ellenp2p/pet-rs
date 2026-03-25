//! CLI/TUI Agent Example using ratatui
//!
//! This example demonstrates the agent-pet-rs framework in a terminal UI
//! using ratatui and crossterm for a professional TUI experience.
//!
//! ## Usage
//!
//! ```bash
//! cargo run --example cli_pet
//! ```

use agent_pet_rs::prelude::*;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use std::{
    io,
    time::{Duration, Instant},
};

// ============================================================
// Game State
// ============================================================

struct PetState {
    name: String,
    hunger: f32,
    health: f32,
    gold: u32,
    mood: Mood,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Mood {
    Happy,
    Neutral,
    Sad,
    Sick,
    Dead,
}

impl std::fmt::Display for Mood {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mood::Happy => write!(f, "Happy"),
            Mood::Neutral => write!(f, "Neutral"),
            Mood::Sad => write!(f, "Sad"),
            Mood::Sick => write!(f, "Sick"),
            Mood::Dead => write!(f, "Dead"),
        }
    }
}

impl PetState {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            hunger: 100.0,
            health: 100.0,
            gold: 100,
            mood: Mood::Happy,
        }
    }

    fn update_mood(&mut self) {
        self.mood = if self.health <= 0.0 {
            Mood::Dead
        } else if self.health < 25.0 {
            Mood::Sick
        } else if self.hunger < 20.0 {
            Mood::Sad
        } else if self.hunger > 60.0 && self.health > 60.0 {
            Mood::Happy
        } else {
            Mood::Neutral
        };
    }

    fn feed(&mut self, amount: f32) {
        self.hunger = (self.hunger + amount).min(100.0);
        self.update_mood();
    }

    fn heal(&mut self, amount: f32) {
        self.health = (self.health + amount).min(100.0);
        self.update_mood();
    }

    fn spend(&mut self, amount: u32) -> bool {
        if self.gold >= amount {
            self.gold -= amount;
            true
        } else {
            false
        }
    }

    fn gain(&mut self, amount: u32) {
        self.gold += amount;
    }

    fn decay(&mut self, delta: f32) {
        self.hunger = (self.hunger - delta * 0.5).max(0.0);
        if self.hunger <= 0.0 {
            self.health = (self.health - delta * 2.0).max(0.0);
        } else if self.hunger < 20.0 {
            self.health = (self.health - delta * 0.5).max(0.0);
        }
        self.update_mood();
    }
}

// ============================================================
// Statistics
// ============================================================

struct Stats {
    purchases: u32,
    heals: u32,
    gold_earned: u32,
}

impl Stats {
    fn new() -> Self {
        Self {
            purchases: 0,
            heals: 0,
            gold_earned: 0,
        }
    }
}

// ============================================================
// App State
// ============================================================

struct App {
    pet: PetState,
    stats: Stats,
    hooks: HookRegistry,
    should_quit: bool,
    last_update: Instant,
    update_interval: Duration,
    messages: Vec<String>,
    vip_level: u32,
    discount_chance: u32,
}

impl App {
    fn new() -> Self {
        let hooks = HookRegistry::default();
        let pet = PetState::new("Buddy");

        Self {
            pet,
            stats: Stats::new(),
            hooks,
            should_quit: false,
            last_update: Instant::now(),
            update_interval: Duration::from_millis(500),
            messages: vec!["Welcome to Virtual Pet!".to_string()],
            vip_level: 1,
            discount_chance: 20,
        }
    }

    fn update(&mut self) {
        let now = Instant::now();
        let delta = now.duration_since(self.last_update).as_secs_f32();
        if now.duration_since(self.last_update) >= self.update_interval {
            self.pet.decay(delta);
            self.last_update = now;
        }
    }

    fn add_message(&mut self, msg: String) {
        self.messages.push(msg);
        if self.messages.len() > 5 {
            self.messages.remove(0);
        }
    }

    fn on_tick(&mut self) {
        self.update();
    }

    fn calculate_discount_chance(&self) -> u32 {
        let mut chance = 20;
        chance += self.vip_level * 5;
        chance += self.stats.purchases * 2;
        chance.min(80)
    }

    fn try_discount(&self) -> Option<u32> {
        let chance = self.calculate_discount_chance();
        let random = (Instant::now().elapsed().as_nanos() % 100) as u32;
        if random < chance {
            let amount = 10 + self.vip_level * 5;
            Some(amount.min(50))
        } else {
            None
        }
    }

    fn record_purchase(&mut self) {
        self.stats.purchases += 1;
        if self.stats.purchases.is_multiple_of(10) {
            self.vip_level += 1;
        }
        self.discount_chance = self.calculate_discount_chance();
    }

    fn on_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                self.should_quit = true;
            }
            KeyCode::Char('1') => {
                let price = 10;
                if let Some(discount) = self.try_discount() {
                    let final_price = (price as f32 * (1.0 - discount as f32 / 100.0)) as u32;
                    if self.pet.spend(final_price) {
                        self.pet.feed(20.0);
                        self.record_purchase();
                        self.add_message(format!(
                            "🎉 {}% discount! Paid {}g",
                            discount, final_price
                        ));
                    } else {
                        self.add_message("Not enough gold!".to_string());
                    }
                } else if self.pet.spend(price) {
                    self.pet.feed(20.0);
                    self.record_purchase();
                    self.add_message("Bought Basic Food! (+20 hunger)".to_string());
                } else {
                    self.add_message("Not enough gold!".to_string());
                }
            }
            KeyCode::Char('2') => {
                let price = 25;
                if let Some(discount) = self.try_discount() {
                    let final_price = (price as f32 * (1.0 - discount as f32 / 100.0)) as u32;
                    if self.pet.spend(final_price) {
                        self.pet.feed(50.0);
                        self.record_purchase();
                        self.add_message(format!(
                            "🎉 {}% discount! Paid {}g",
                            discount, final_price
                        ));
                    } else {
                        self.add_message("Not enough gold!".to_string());
                    }
                } else if self.pet.spend(price) {
                    self.pet.feed(50.0);
                    self.record_purchase();
                    self.add_message("Bought Premium Food! (+50 hunger)".to_string());
                } else {
                    self.add_message("Not enough gold!".to_string());
                }
            }
            KeyCode::Char('3') => {
                let price = 50;
                if let Some(discount) = self.try_discount() {
                    let final_price = (price as f32 * (1.0 - discount as f32 / 100.0)) as u32;
                    if self.pet.spend(final_price) {
                        self.pet.heal(30.0);
                        self.record_purchase();
                        self.add_message(format!(
                            "🎉 {}% discount! Paid {}g",
                            discount, final_price
                        ));
                    } else {
                        self.add_message("Not enough gold!".to_string());
                    }
                } else if self.pet.spend(price) {
                    self.pet.heal(30.0);
                    self.record_purchase();
                    self.add_message("Bought Elixir! (+30 health)".to_string());
                } else {
                    self.add_message("Not enough gold!".to_string());
                }
            }
            KeyCode::Char('f') | KeyCode::Char('F') => {
                if self.pet.spend(10) {
                    self.pet.feed(20.0);
                    self.stats.purchases += 1;
                    self.add_message("Fed pet! (+20 hunger)".to_string());
                } else {
                    self.add_message("Not enough gold!".to_string());
                }
            }
            KeyCode::Char('h') | KeyCode::Char('H') => {
                self.pet.heal(15.0);
                self.stats.heals += 1;
                self.add_message("Healed pet! (+15 health)".to_string());
            }
            KeyCode::Char('g') | KeyCode::Char('G') => {
                self.pet.gain(50);
                self.stats.gold_earned += 50;
                self.add_message("Gained 50 gold!".to_string());
            }
            _ => {}
        }
    }
}

// ============================================================
// UI Rendering
// ============================================================

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(10),   // Main content
            Constraint::Length(3), // Controls
        ])
        .split(f.area());

    // Title
    let title = Paragraph::new("VIRTUAL PET - CLI/TUI Demo")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Main content split into left (Pet Status) and right (Shop)
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    // Pet Status (Left)
    let pet_status = create_pet_status(app);
    f.render_widget(pet_status, main_chunks[0]);

    // Shop (Right)
    let shop = create_shop(app);
    f.render_widget(shop, main_chunks[1]);

    // Controls (Bottom)
    let controls = create_controls();
    f.render_widget(controls, chunks[2]);
}

fn create_pet_status(app: &App) -> Paragraph<'_> {
    let pet_info = vec![
        Line::from(vec![
            Span::styled("Name: ", Style::default().fg(Color::White)),
            Span::styled(
                app.pet.name.clone(),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Health: ", Style::default().fg(Color::White)),
            Span::styled(
                format!("{:.0}%", app.pet.health),
                Style::default()
                    .fg(if app.pet.health > 50.0 {
                        Color::Green
                    } else if app.pet.health > 25.0 {
                        Color::Yellow
                    } else {
                        Color::Red
                    })
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Hunger: ", Style::default().fg(Color::White)),
            Span::styled(
                format!("{:.0}%", app.pet.hunger),
                Style::default()
                    .fg(if app.pet.hunger > 50.0 {
                        Color::Green
                    } else if app.pet.hunger > 25.0 {
                        Color::Yellow
                    } else {
                        Color::Red
                    })
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Mood: ", Style::default().fg(Color::White)),
            Span::styled(
                format!("{}", app.pet.mood),
                Style::default()
                    .fg(match app.pet.mood {
                        Mood::Happy => Color::Green,
                        Mood::Neutral => Color::Yellow,
                        Mood::Sad => Color::Blue,
                        Mood::Sick => Color::Magenta,
                        Mood::Dead => Color::Red,
                    })
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Gold: ", Style::default().fg(Color::White)),
            Span::styled(
                format!("{}", app.pet.gold),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("VIP: ", Style::default().fg(Color::White)),
            Span::styled(
                format!("Lv.{}", app.vip_level),
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("  Disc: ", Style::default().fg(Color::White)),
            Span::styled(
                format!("{}%", app.discount_chance),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Stats",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::styled(
            format!("Purchases: {}", app.stats.purchases),
            Style::default().fg(Color::Cyan),
        )]),
        Line::from(vec![Span::styled(
            format!("Heals: {}", app.stats.heals),
            Style::default().fg(Color::Green),
        )]),
        Line::from(vec![Span::styled(
            format!("Gold Earned: {}", app.stats.gold_earned),
            Style::default().fg(Color::Yellow),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Messages:",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )]),
    ];

    let mut all_lines = pet_info;
    for msg in &app.messages {
        all_lines.push(Line::from(vec![Span::styled(
            format!("  {}", msg),
            Style::default().fg(Color::Gray),
        )]));
    }

    Paragraph::new(all_lines).block(
        Block::default()
            .title("PET STATUS")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    )
}

fn create_shop(_app: &App) -> Paragraph<'_> {
    let items = vec![
        Line::from(vec![Span::styled(
            "Available Items:",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("[1] Basic Food   ", Style::default().fg(Color::Green)),
            Span::styled("- 10g  ", Style::default().fg(Color::Yellow)),
            Span::styled("(+20 hunger)", Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled("[2] Premium Food ", Style::default().fg(Color::Blue)),
            Span::styled("- 25g  ", Style::default().fg(Color::Yellow)),
            Span::styled("(+50 hunger)", Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled("[3] Elixir       ", Style::default().fg(Color::Magenta)),
            Span::styled("- 50g  ", Style::default().fg(Color::Yellow)),
            Span::styled("(+30 health)", Style::default().fg(Color::Green)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Quick Actions:",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("[F] ", Style::default().fg(Color::Green)),
            Span::styled(
                "Quick Food - 10g (+20 hunger)",
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::styled("[H] ", Style::default().fg(Color::Green)),
            Span::styled("Heal (+15 health)", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("[G] ", Style::default().fg(Color::Yellow)),
            Span::styled("Gain Gold (+50g)", Style::default().fg(Color::White)),
        ]),
    ];

    Paragraph::new(items).block(
        Block::default()
            .title("SHOP")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow)),
    )
}

fn create_controls() -> Paragraph<'static> {
    let controls = vec![Line::from(vec![
        Span::styled(
            "[1-3] ",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Buy Items  ", Style::default().fg(Color::White)),
        Span::styled(
            "[F] ",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Quick Food  ", Style::default().fg(Color::White)),
        Span::styled(
            "[H] ",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Heal  ", Style::default().fg(Color::White)),
        Span::styled(
            "[G] ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Gold  ", Style::default().fg(Color::White)),
        Span::styled(
            "[Q] ",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ),
        Span::styled("Quit", Style::default().fg(Color::White)),
    ])];

    Paragraph::new(controls).block(
        Block::default()
            .title("CONTROLS")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White)),
    )
}

// ============================================================
// Main Loop
// ============================================================

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new();

    // Main loop
    let tick_rate = Duration::from_millis(250);
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| ui(f, &app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    app.on_key(key.code);
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }

        if app.should_quit {
            break;
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
