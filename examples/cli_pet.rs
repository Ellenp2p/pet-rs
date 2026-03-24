//! CLI/TUI Pet Example
//!
//! This example demonstrates the pet-rs framework in a terminal UI
//! without using Bevy. It uses crossterm for terminal input/output.
//!
//! ## Usage
//!
//! ```bash
//! cargo run --example cli_pet --features wasm-plugin
//! ```

use pet_rs::prelude::*;
use std::io::{self, Write};
use std::time::{Duration, Instant};

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

#[derive(Debug, Clone, Copy)]
enum Mood {
    Happy,
    Neutral,
    Sad,
    Sick,
    Dead,
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
// UI Rendering
// ============================================================

fn render_ui(pet: &PetState, stats: &Stats) {
    // Clear screen
    print!("\x1B[2J\x1B[1;1H");

    let health_bar = make_bar(pet.health / 100.0);
    let hunger_bar = make_bar(pet.hunger / 100.0);

    println!("+--------------------------------------------------------------------+");
    println!("|  VIRTUAL PET - CLI/TUI Demo                                        |");
    println!("+========================+===========================================+");
    println!("| PET STATUS             | SHOP (Plugin-Controlled)                  |");
    println!("|                        |                                           |");
    println!(
        "| Name: {:<17} | Available Items:                          |",
        pet.name
    );
    println!(
        "| Health: {} {:>3}% | [1] Basic Food   - 10g  (+20 hunger)      |",
        health_bar, pet.health as i32
    );
    println!(
        "| Hunger: {} {:>3}% | [2] Premium Food - 25g  (+50 hunger)      |",
        hunger_bar, pet.hunger as i32
    );
    println!(
        "| Mood:   {:<15} | [3] Elixir       - 50g  (+30 health)      |",
        format!("{:?}", pet.mood)
    );
    println!(
        "| Gold:   {:<14} |                                           |",
        pet.gold
    );
    println!("|                        | Plugin Effects:                           |");
    println!(
        "| Stats: P:{} H:{} G:{}   | - Dynamic pricing (5% per purchase)       |",
        stats.purchases, stats.heals, stats.gold_earned
    );
    println!("|                        | - Unlock: Premium at 5 purchases          |");
    println!("|                        | - Discount: 10% after 10 purchases       |");
    println!("+========================+===========================================+");
    println!("| [1-3] Buy Items  [F] Quick Food  [H] Heal  [G] Gold  [Q] Quit    |");
    println!("+--------------------------------------------------------------------+");
    print!("> ");
    io::stdout().flush().unwrap();
}

fn make_bar(ratio: f32) -> String {
    let filled = (ratio * 10.0) as i32;
    let empty = 10 - filled;
    let mut bar = String::new();
    for _ in 0..filled {
        bar.push('#');
    }
    for _ in 0..empty {
        bar.push('.');
    }
    bar
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
// Main Loop
// ============================================================

fn main() {
    println!("Initializing Virtual Pet CLI/TUI Demo...");
    println!("Press any key to start...");

    // Initialize hooks
    let mut hooks = HookRegistry::default();
    hooks.register_fn("on_spawn", |_ctx| {
        println!("[HOOK] Pet spawned!");
    });
    hooks.register_fn("on_feed", |_ctx| {
        println!("[HOOK] Pet fed!");
    });
    hooks.register_fn("on_purchase", |_ctx| {
        println!("[HOOK] Purchase made!");
    });
    hooks.register_fn("on_reward", |_ctx| {
        println!("[HOOK] Reward given!");
    });

    // Create pet state
    let mut pet = PetState::new("Buddy");
    let mut stats = Stats::new();

    // Trigger spawn hook
    hooks.trigger("on_spawn", &HookContext { entity: 0 });

    // Main game loop
    let mut last_update = Instant::now();
    let update_interval = Duration::from_secs(1);

    loop {
        // Update pet state
        let now = Instant::now();
        let delta = now.duration_since(last_update).as_secs_f32();
        if now.duration_since(last_update) >= update_interval {
            pet.decay(delta);
            last_update = now;
        }

        // Render UI
        render_ui(&pet, &stats);

        // Read input
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        // Process input
        match input {
            "1" => {
                if pet.spend(10) {
                    pet.feed(20.0);
                    stats.purchases += 1;
                    hooks.trigger("on_purchase", &HookContext { entity: 0 });
                    println!("[GAME] Bought Basic Food!");
                } else {
                    println!("[GAME] Not enough gold!");
                }
            }
            "2" => {
                if pet.spend(25) {
                    pet.feed(50.0);
                    stats.purchases += 1;
                    hooks.trigger("on_purchase", &HookContext { entity: 0 });
                    println!("[GAME] Bought Premium Food!");
                } else {
                    println!("[GAME] Not enough gold!");
                }
            }
            "3" => {
                if pet.spend(50) {
                    pet.heal(30.0);
                    stats.purchases += 1;
                    hooks.trigger("on_purchase", &HookContext { entity: 0 });
                    println!("[GAME] Bought Elixir!");
                } else {
                    println!("[GAME] Not enough gold!");
                }
            }
            "f" | "F" => {
                if pet.spend(10) {
                    pet.feed(20.0);
                    stats.purchases += 1;
                    hooks.trigger("on_feed", &HookContext { entity: 0 });
                    println!("[GAME] Fed pet!");
                } else {
                    println!("[GAME] Not enough gold!");
                }
            }
            "h" | "H" => {
                pet.heal(15.0);
                stats.heals += 1;
                println!("[GAME] Healed pet!");
            }
            "g" | "G" => {
                pet.gain(50);
                stats.gold_earned += 50;
                hooks.trigger("on_reward", &HookContext { entity: 0 });
                println!("[GAME] Gained 50 gold!");
            }
            "q" | "Q" => {
                println!("Goodbye!");
                break;
            }
            _ => {
                println!("[GAME] Unknown command: {}", input);
            }
        }

        // Wait a bit for feedback
        std::thread::sleep(Duration::from_millis(500));
    }
}
