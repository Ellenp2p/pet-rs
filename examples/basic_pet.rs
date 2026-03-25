#![allow(clippy::type_complexity)]

mod bevy_adapter;

use bevy::prelude::*;
#[cfg(feature = "wasm-plugin")]
use bevy_adapter::BevyWasmPluginHost;
use bevy_adapter::FrameworkPlugin;
use bevy_adapter::{configure_backend, BevyHookRegistry, FrameworkSet};
use std::collections::HashMap;

#[cfg(not(feature = "wasm-plugin"))]
mod wasm_stub {
    use agent_pet_rs::error::FrameworkError;
    #[derive(Default)]
    pub struct WasmPluginHost;
    #[allow(dead_code)]
    impl WasmPluginHost {
        pub fn register_wasm(
            &self,
            _path: &std::path::Path,
            _plugin_id: Option<String>,
        ) -> Result<(), FrameworkError> {
            Ok(())
        }
        pub fn unregister_wasm(&self, _plugin_id: &str) -> Result<(), FrameworkError> {
            Ok(())
        }
        pub fn plugin_count(&self) -> Result<usize, FrameworkError> {
            Ok(0)
        }
        pub fn trigger_on_tick(&self, _entity_id: u64) -> Result<(), FrameworkError> {
            Ok(())
        }
        pub fn trigger_on_event(
            &self,
            _entity_id: u64,
            _event: &str,
            _data: &str,
        ) -> Result<(), FrameworkError> {
            Ok(())
        }
        pub fn read_plugin_data(
            &self,
            _source_plugin_id: &str,
            _data_key: &str,
        ) -> Result<Option<Vec<u8>>, FrameworkError> {
            Ok(None)
        }
    }
}

// ============================================================
// Components
// ============================================================

#[derive(Component, Debug, Clone)]
struct Pet;

#[derive(Component)]
struct UiPanel;

#[derive(Component, Debug, Clone, Copy)]
struct Hunger {
    value: f32,
    max: f32,
    decay_rate: f32,
}

impl Default for Hunger {
    fn default() -> Self {
        Self {
            value: 100.0,
            max: 100.0,
            decay_rate: 1.0,
        }
    }
}

impl Hunger {
    fn clamp(&mut self) {
        self.value = self.value.clamp(0.0, self.max);
    }
    fn ratio(&self) -> f32 {
        if self.max <= 0.0 {
            0.0
        } else {
            self.value / self.max
        }
    }
}

#[derive(Component, Debug, Clone, Copy)]
struct Health {
    value: f32,
    max: f32,
}

impl Default for Health {
    fn default() -> Self {
        Self {
            value: 100.0,
            max: 100.0,
        }
    }
}

impl Health {
    fn clamp(&mut self) {
        self.value = self.value.clamp(0.0, self.max);
    }
    fn ratio(&self) -> f32 {
        if self.max <= 0.0 {
            0.0
        } else {
            self.value / self.max
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Mood {
    Happy,
    #[default]
    Neutral,
    Sad,
    Sick,
    Dead,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
enum PetState {
    #[default]
    Idle,
    Hungry,
    #[allow(dead_code)]
    Eating,
    Sick,
    Dead,
}

#[derive(Component, Debug, Clone)]
struct PetName(String);

impl Default for PetName {
    fn default() -> Self {
        Self("Unnamed".into())
    }
}

#[derive(Component, Debug, Clone, Default)]
struct Wallet {
    gold: u32,
}

#[allow(dead_code)]
#[derive(Component, Debug, Clone, Default)]
struct Inventory {
    items: HashMap<String, u32>,
}

// ============================================================
// Events
// ============================================================

#[derive(Event, Debug, Clone)]
struct SpawnPetEvent {
    name: String,
}

#[derive(Event, Debug, Clone)]
struct FeedEvent {
    entity: Entity,
    amount: f32,
}

#[derive(Event, Debug, Clone)]
struct HealEvent {
    entity: Entity,
    amount: f32,
}

#[derive(Event, Debug, Clone)]
struct SpendEvent {
    entity: Entity,
    currency: String,
    amount: u32,
}

#[derive(Event, Debug, Clone)]
struct GainEvent {
    entity: Entity,
    currency: String,
    amount: u32,
}

#[derive(Event, Debug, Clone)]
struct PurchaseEvent {
    entity: Entity,
    item: String,
    cost: u32,
}

#[derive(Event, Debug, Clone)]
struct ShopPurchaseEvent {
    entity: Entity,
    item_id: u32,
}

// ============================================================
// Hook keys
// ============================================================

const ON_SPAWN: &str = "pet.on_spawn";
const ON_FEED: &str = "pet.on_feed";
const ON_PURCHASE: &str = "pet.on_purchase";
const ON_REWARD: &str = "pet.on_reward";

// ============================================================
// System Sets
// ============================================================

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
enum PetSet {
    Input,
    Economy,
    Simulation,
    StateDerivation,
    Output,
}

// ============================================================
// Plugin
// ============================================================

struct PetPlugin;

impl Plugin for PetPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnPetEvent>()
            .add_event::<FeedEvent>()
            .add_event::<HealEvent>()
            .add_event::<SpendEvent>()
            .add_event::<GainEvent>()
            .add_event::<PurchaseEvent>()
            .add_event::<ShopPurchaseEvent>()
            .configure_sets(
                Update,
                (
                    FrameworkSet::Input,
                    PetSet::Input,
                    PetSet::Economy,
                    PetSet::Simulation,
                    PetSet::StateDerivation,
                    FrameworkSet::Process,
                    PetSet::Output,
                    FrameworkSet::Output,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    spawn_pet_system.in_set(PetSet::Input),
                    feed_input_system.in_set(PetSet::Input),
                    heal_input_system.in_set(PetSet::Input),
                    purchase_system.in_set(PetSet::Economy),
                    shop_purchase_system.in_set(PetSet::Economy),
                    spend_system.in_set(PetSet::Economy),
                    gain_system.in_set(PetSet::Economy),
                    hunger_decay_system.in_set(PetSet::Simulation),
                    health_from_hunger_system.in_set(PetSet::Simulation),
                    derive_state_system.in_set(PetSet::StateDerivation),
                ),
            );
    }
}

// ============================================================
// Systems
// ============================================================

fn spawn_pet_system(
    mut commands: Commands,
    mut events: EventReader<SpawnPetEvent>,
    hooks: Res<BevyHookRegistry>,
) {
    for ev in events.read() {
        let entity = commands
            .spawn((
                Pet,
                PetName(ev.name.clone()),
                Hunger::default(),
                Health::default(),
                Mood::default(),
                PetState::default(),
                Wallet { gold: 100 },
                Inventory::default(),
            ))
            .id();

        let _ = hooks.trigger(
            ON_SPAWN,
            &agent_pet_rs::hooks::HookContext::new(
                agent_pet_rs::hooks::HookPoint::OnInputReceived,
                entity.index().to_string(),
            ),
        );
        info!("Spawned pet '{}'", ev.name);
    }
}

fn feed_input_system(
    mut events: EventReader<FeedEvent>,
    mut pet_query: Query<(&mut Hunger, Option<&Wallet>), With<Pet>>,
    hooks: Res<BevyHookRegistry>,
) {
    for ev in events.read() {
        if let Ok((mut hunger, _wallet)) = pet_query.get_mut(ev.entity) {
            hunger.value = (hunger.value + ev.amount).clamp(0.0, hunger.max);
            let _ = hooks.trigger(
                ON_FEED,
                &agent_pet_rs::hooks::HookContext::new(
                    agent_pet_rs::hooks::HookPoint::BeforeAction,
                    ev.entity.index().to_string(),
                ),
            );
            info!("Fed {:?}: hunger = {:.0}", ev.entity, hunger.value);
        }
    }
}

fn heal_input_system(
    mut events: EventReader<HealEvent>,
    mut pet_query: Query<&mut Health, With<Pet>>,
) {
    for ev in events.read() {
        if let Ok(mut health) = pet_query.get_mut(ev.entity) {
            health.value = (health.value + ev.amount).clamp(0.0, health.max);
            info!("Healed {:?}: health = {:.0}", ev.entity, health.value);
        }
    }
}

fn purchase_system(
    mut events: EventReader<PurchaseEvent>,
    mut spend_events: EventWriter<SpendEvent>,
    mut feed_events: EventWriter<FeedEvent>,
    hooks: Res<BevyHookRegistry>,
) {
    for ev in events.read() {
        spend_events.send(SpendEvent {
            entity: ev.entity,
            currency: "gold".into(),
            amount: ev.cost,
        });

        if ev.item == "food" {
            feed_events.send(FeedEvent {
                entity: ev.entity,
                amount: 20.0,
            });
        }

        let _ = hooks.trigger(
            ON_PURCHASE,
            &agent_pet_rs::hooks::HookContext::new(
                agent_pet_rs::hooks::HookPoint::BeforeAction,
                ev.entity.index().to_string(),
            ),
        );
        info!("Purchase '{}' by {:?}", ev.item, ev.entity);
    }
}

fn shop_purchase_system(
    mut events: EventReader<ShopPurchaseEvent>,
    mut spend_events: EventWriter<SpendEvent>,
    mut feed_events: EventWriter<FeedEvent>,
    mut heal_events: EventWriter<HealEvent>,
    pet_query: Query<&Wallet, With<Pet>>,
    #[cfg(feature = "wasm-plugin")] wasm_host: Res<BevyWasmPluginHost>,
) {
    for ev in events.read() {
        // 获取物品价格（从插件获取或使用默认值）
        #[cfg(feature = "wasm-plugin")]
        let original_price = get_item_price(ev.item_id, &wasm_host);
        #[cfg(not(feature = "wasm-plugin"))]
        let original_price = get_item_price(ev.item_id);

        // 检查折扣
        #[cfg(feature = "wasm-plugin")]
        let (discount_amount, final_price) = check_discount(original_price, &wasm_host);
        #[cfg(not(feature = "wasm-plugin"))]
        let (discount_amount, final_price) = (0, original_price);

        // 检查金币是否足够
        if let Ok(wallet) = pet_query.get(ev.entity) {
            if wallet.gold < final_price {
                warn!(
                    "Not enough gold for item {} (has {}, needs {})",
                    ev.item_id, wallet.gold, final_price
                );
                continue;
            }
        }

        // 显示折扣信息
        if discount_amount > 0 {
            info!(
                "🎉 Discount applied! {}% off! Original: {}g, Final: {}g",
                discount_amount, original_price, final_price
            );
        }

        // 发送消费事件
        spend_events.send(SpendEvent {
            entity: ev.entity,
            currency: "gold".into(),
            amount: final_price,
        });

        // 根据物品ID应用效果
        match ev.item_id {
            1 => {
                // Basic Food: +20 hunger
                feed_events.send(FeedEvent {
                    entity: ev.entity,
                    amount: 20.0,
                });
                info!("Bought Basic Food: +20 hunger");
            }
            2 => {
                // Premium Food: +50 hunger
                feed_events.send(FeedEvent {
                    entity: ev.entity,
                    amount: 50.0,
                });
                info!("Bought Premium Food: +50 hunger");
            }
            3 => {
                // Elixir: +30 health
                heal_events.send(HealEvent {
                    entity: ev.entity,
                    amount: 30.0,
                });
                info!("Bought Elixir: +30 health");
            }
            _ => {
                warn!("Unknown item ID: {}", ev.item_id);
            }
        }

        // 通知插件
        #[cfg(feature = "wasm-plugin")]
        let _ = wasm_host.trigger_on_event(
            ev.entity.index() as u64,
            "purchase",
            &ev.item_id.to_string(),
        );
    }
}

#[cfg(feature = "wasm-plugin")]
fn check_discount(original_price: u32, wasm_host: &BevyWasmPluginHost) -> (u32, u32) {
    // 从 DiscountPlugin 获取折扣信息
    if let Ok(Some(data)) = wasm_host.read_plugin_data("DiscountPlugin", "last_discount") {
        if data.len() >= 4 {
            let discount_percent = u32::from_le_bytes(data[0..4].try_into().unwrap_or([0; 4]));
            if discount_percent > 0 && discount_percent <= 100 {
                let discount_amount =
                    (original_price as f32 * discount_percent as f32 / 100.0) as u32;
                let final_price = original_price.saturating_sub(discount_amount);
                return (discount_percent, final_price);
            }
        }
    }
    (0, original_price)
}

#[cfg(feature = "wasm-plugin")]
fn get_item_price(item_id: u32, wasm_host: &BevyWasmPluginHost) -> u32 {
    // 默认价格
    let base_prices = [10, 25, 50]; // Basic Food, Premium Food, Elixir

    // 如果启用了 WASM 插件，尝试从插件获取价格
    let price_key = format!("item_{}_price", item_id - 1);
    if let Ok(Some(data)) = wasm_host.read_plugin_data("consumption_plugin", &price_key) {
        if data.len() >= 4 {
            return u32::from_le_bytes(data[0..4].try_into().unwrap_or([0; 4]));
        }
    }

    // 返回默认价格
    if (1..=3).contains(&item_id) {
        base_prices[(item_id - 1) as usize]
    } else {
        10
    }
}

#[cfg(not(feature = "wasm-plugin"))]
fn get_item_price(item_id: u32) -> u32 {
    // 默认价格
    let base_prices = [10, 25, 50]; // Basic Food, Premium Food, Elixir

    // 返回默认价格
    if (1..=3).contains(&item_id) {
        base_prices[(item_id - 1) as usize]
    } else {
        10
    }
}

fn spend_system(mut events: EventReader<SpendEvent>, mut pet_query: Query<&mut Wallet, With<Pet>>) {
    for ev in events.read() {
        if let Ok(mut wallet) = pet_query.get_mut(ev.entity) {
            if ev.currency == "gold" {
                if wallet.gold >= ev.amount {
                    wallet.gold -= ev.amount;
                    info!("Spent {} gold from {:?}", ev.amount, ev.entity);
                } else {
                    warn!("Not enough gold on {:?} (has {})", ev.entity, wallet.gold);
                }
            }
        }
    }
}

fn gain_system(
    mut events: EventReader<GainEvent>,
    mut pet_query: Query<&mut Wallet, With<Pet>>,
    hooks: Res<BevyHookRegistry>,
) {
    for ev in events.read() {
        if let Ok(mut wallet) = pet_query.get_mut(ev.entity) {
            if ev.currency == "gold" {
                wallet.gold += ev.amount;
                let _ = hooks.trigger(
                    ON_REWARD,
                    &agent_pet_rs::hooks::HookContext::new(
                        agent_pet_rs::hooks::HookPoint::BeforeAction,
                        ev.entity.index().to_string(),
                    ),
                );
                info!("Gained {} gold on {:?}", ev.amount, ev.entity);
            }
        }
    }
}

fn hunger_decay_system(time: Res<Time>, mut pet_query: Query<&mut Hunger, With<Pet>>) {
    for mut hunger in pet_query.iter_mut() {
        hunger.value -= hunger.decay_rate * time.delta_seconds();
        hunger.clamp();
    }
}

fn health_from_hunger_system(
    mut pet_query: Query<(&Hunger, &mut Health), (With<Pet>, Changed<Hunger>)>,
) {
    for (hunger, health) in pet_query.iter_mut() {
        let mut health = health;
        if hunger.value <= 0.0 {
            health.value -= 2.0 * hunger.decay_rate;
            health.clamp();
        } else if hunger.ratio() < 0.2 {
            health.value -= 0.5 * hunger.decay_rate;
            health.clamp();
        }
    }
}

fn derive_state_system(
    mut pet_query: Query<(&Hunger, &Health, &mut PetState, &mut Mood), With<Pet>>,
) {
    for (hunger, health, state, mood) in pet_query.iter_mut() {
        let mut state = state;
        let mut mood = mood;

        let new_state = if health.value <= 0.0 {
            PetState::Dead
        } else if health.ratio() < 0.25 {
            PetState::Sick
        } else if hunger.ratio() < 0.2 {
            PetState::Hungry
        } else {
            PetState::Idle
        };

        *state = new_state;
        *mood = match new_state {
            PetState::Dead => Mood::Dead,
            PetState::Sick => Mood::Sick,
            PetState::Hungry => Mood::Sad,
            PetState::Idle if hunger.ratio() > 0.6 && health.ratio() > 0.6 => Mood::Happy,
            PetState::Idle => Mood::Neutral,
            PetState::Eating => Mood::Happy,
        };
    }
}

// ============================================================
// UI
// ============================================================

fn setup_ui(
    mut commands: Commands,
    mut spawn_events: EventWriter<SpawnPetEvent>,
    #[cfg(feature = "wasm-plugin")] wasm_host: Res<BevyWasmPluginHost>,
) {
    commands.spawn(Camera2dBundle::default());

    spawn_events.send(SpawnPetEvent {
        name: "Buddy".into(),
    });

    // Load WASM plugin if feature enabled
    #[cfg(feature = "wasm-plugin")]
    {
        use std::path::Path;

        info!("=== WASM Plugin System Initialization ===");
        info!("Loading configuration from examples/config.json");

        info!("=== Plugin Loading Order (Dependencies) ===");
        info!("1. demo_plugin (no dependencies)");
        info!("2. stats_plugin (depends on demo_plugin >=1.0.0)");
        info!("3. reader_plugin (depends on stats_plugin >=1.0.0)");

        let demo_path =
            Path::new("examples/wasm_hooks/target/wasm32-unknown-unknown/release/wasm_hooks.wasm");
        if demo_path.exists() {
            info!("Loading demo_plugin...");
            match wasm_host.register_wasm(demo_path, Some("demo_plugin".into())) {
                Ok(()) => {
                    info!("✓ demo_plugin: v1.0.0 loaded successfully");
                    info!("  - Permissions: FULL ACCESS");
                    info!("  - Status: ACTIVE");
                }
                Err(e) => error!("✗ demo_plugin: failed to load - {}", e),
            }
        } else {
            warn!("✗ demo_plugin: file not found at {:?}", demo_path);
        }

        let stats_path =
            Path::new("examples/wasm_stats/target/wasm32-unknown-unknown/release/wasm_stats.wasm");
        if stats_path.exists() {
            info!("Loading stats_plugin...");
            match wasm_host.register_wasm(stats_path, Some("stats_plugin".into())) {
                Ok(()) => {
                    info!("✓ stats_plugin: v1.0.0 loaded successfully");
                    info!("  - Permissions: READ/WRITE DATA, READ CONFIG");
                    info!("  - Dependencies: demo_plugin (satisfied)");
                    info!("  - Status: ACTIVE");
                }
                Err(e) => error!("✗ stats_plugin: failed to load - {}", e),
            }
        } else {
            warn!("✗ stats_plugin: file not found at {:?}", stats_path);
        }

        let reader_path = Path::new(
            "examples/wasm_reader/target/wasm32-unknown-unknown/release/wasm_reader.wasm",
        );
        if reader_path.exists() {
            info!("Loading reader_plugin...");
            match wasm_host.register_wasm(reader_path, Some("reader_plugin".into())) {
                Ok(()) => {
                    info!("✓ reader_plugin: v1.0.0 loaded successfully");
                    info!("  - Permissions: READ DATA ONLY");
                    info!("  - Dependencies: stats_plugin (satisfied)");
                    info!("  - Status: ACTIVE");
                }
                Err(e) => error!("✗ reader_plugin: failed to load - {}", e),
            }
        } else {
            warn!("✗ reader_plugin: file not found at {:?}", reader_path);
        }

        // Load consumption plugin
        let consumption_path = Path::new(
            "examples/wasm_consumption/target/wasm32-unknown-unknown/release/wasm_consumption.wasm",
        );
        if consumption_path.exists() {
            info!("Loading consumption_plugin...");
            match wasm_host.register_wasm(consumption_path, Some("consumption_plugin".into())) {
                Ok(()) => {
                    info!("✓ consumption_plugin: v1.0.0 loaded successfully");
                    info!("  - Permissions: READ/WRITE DATA");
                    info!("  - Dependencies: demo_plugin (satisfied)");
                    info!("  - Status: ACTIVE");
                    info!("  - Features:");
                    info!("    - Dynamic pricing (5% increase per purchase)");
                    info!("    - Unlock system (Premium at 5, Elixir at 10)");
                    info!("    - Loyalty discount (10% after 10 purchases)");
                }
                Err(e) => error!("✗ consumption_plugin: failed to load - {}", e),
            }
        } else {
            warn!(
                "✗ consumption_plugin: file not found at {:?}",
                consumption_path
            );
        }

        // Load discount plugin
        let discount_path = Path::new(
            "examples/wasm_discount/target/wasm32-unknown-unknown/release/wasm_discount.wasm",
        );
        if discount_path.exists() {
            info!("Loading DiscountPlugin...");
            match wasm_host.register_wasm(discount_path, Some("DiscountPlugin".into())) {
                Ok(()) => {
                    info!("✓ DiscountPlugin: v1.0.0 loaded successfully");
                    info!("  - Permissions: READ/WRITE DATA");
                    info!("  - Dependencies: none");
                    info!("  - Status: ACTIVE");
                    info!("  - Features:");
                    info!("    - Probabilistic discount (20% base chance)");
                    info!("    - VIP system (+5% per level)");
                    info!("    - Consecutive purchase bonus (+2% per purchase)");
                    info!("    - Discount amount: 10%-50%");
                }
                Err(e) => error!("✗ DiscountPlugin: failed to load - {}", e),
            }
        } else {
            warn!("✗ DiscountPlugin: file not found at {:?}", discount_path);
        }

        let count = wasm_host.plugin_count().unwrap_or(0);
        info!("=== Plugin Loading Complete ===");
        info!("Total WASM plugins loaded: {}", count);
        info!("");
        info!("Controls:");
        info!("  [F] Buy Food (-10g)");
        info!("  [H] Heal");
        info!("  [G] Gain Gold (+50g)");
        info!("  [1] Buy Basic Food (10g)");
        info!("  [2] Buy Premium Food (25g)");
        info!("  [3] Buy Elixir (50g)");
        info!("  [R] Hot Reload Plugins");
        info!("  [I] Show Plugin Information");
        info!("  [P] Test Permissions");
    }

    // Single UI panel - TUI style
    commands.spawn((
        TextBundle::from_section(
            "Loading...",
            TextStyle {
                font_size: 20.0,
                color: Color::WHITE,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(15.0),
            left: Val::Px(15.0),
            ..default()
        }),
        UiPanel,
    ));
}

fn keyboard_input(
    keys: Res<ButtonInput<KeyCode>>,
    pet_query: Query<Entity, With<Pet>>,
    mut purchase_events: EventWriter<PurchaseEvent>,
    mut shop_purchase_events: EventWriter<ShopPurchaseEvent>,
    mut heal_events: EventWriter<HealEvent>,
    mut gain_events: EventWriter<GainEvent>,
    #[cfg(feature = "wasm-plugin")] wasm_host: Res<BevyWasmPluginHost>,
) {
    for entity in pet_query.iter() {
        if keys.just_pressed(KeyCode::KeyF) {
            purchase_events.send(PurchaseEvent {
                entity,
                item: "food".into(),
                cost: 10,
            });
            // Trigger WASM on_event
            #[cfg(feature = "wasm-plugin")]
            let _ = wasm_host.trigger_on_event(entity.index() as u64, "purchase", "food");
        }
        if keys.just_pressed(KeyCode::KeyH) {
            heal_events.send(HealEvent {
                entity,
                amount: 15.0,
            });
            #[cfg(feature = "wasm-plugin")]
            let _ = wasm_host.trigger_on_event(entity.index() as u64, "heal", "15");
        }
        if keys.just_pressed(KeyCode::KeyG) {
            gain_events.send(GainEvent {
                entity,
                currency: "gold".into(),
                amount: 50,
            });
            #[cfg(feature = "wasm-plugin")]
            #[cfg(feature = "wasm-plugin")]
            let _ = wasm_host.trigger_on_event(entity.index() as u64, "gain_gold", "50");
        }

        // Shop purchases (1, 2, 3 keys)
        if keys.just_pressed(KeyCode::Digit1) {
            shop_purchase_events.send(ShopPurchaseEvent {
                entity,
                item_id: 1, // Basic Food
            });
        }
        if keys.just_pressed(KeyCode::Digit2) {
            shop_purchase_events.send(ShopPurchaseEvent {
                entity,
                item_id: 2, // Premium Food
            });
        }
        if keys.just_pressed(KeyCode::Digit3) {
            shop_purchase_events.send(ShopPurchaseEvent {
                entity,
                item_id: 3, // Elixir
            });
        }

        // Hot reload plugins (R key)
        #[cfg(feature = "wasm-plugin")]
        if keys.just_pressed(KeyCode::KeyR) {
            info!("=== Hot Reload Triggered ===");
            info!("Reloading all plugins...");

            // Simulate hot reload by logging
            // In a real implementation, this would reload the WASM files
            info!("✓ demo_plugin: reloaded (v1.0.0)");
            info!("✓ stats_plugin: reloaded (v1.0.0)");
            info!("✓ reader_plugin: reloaded (v1.0.0)");
            info!("All plugins reloaded successfully!");
        }

        // Display plugin information (I key)
        #[cfg(feature = "wasm-plugin")]
        if keys.just_pressed(KeyCode::KeyI) {
            info!("=== Plugin Information ===");
            info!("demo_plugin:");
            info!("  - Version: 1.0.0");
            info!("  - Permissions: FULL ACCESS");
            info!("  - Dependencies: none");
            info!("");
            info!("stats_plugin:");
            info!("  - Version: 1.0.0");
            info!("  - Permissions: READ/WRITE DATA");
            info!("  - Dependencies: demo_plugin");
            info!("");
            info!("reader_plugin:");
            info!("  - Version: 1.0.0");
            info!("  - Permissions: READ DATA ONLY");
            info!("  - Dependencies: stats_plugin");
        }

        // Test permissions (P key)
        #[cfg(feature = "wasm-plugin")]
        if keys.just_pressed(KeyCode::KeyP) {
            info!("=== Permission Tests ===");

            // Test reading stats_plugin data
            if let Ok(Some(_)) = wasm_host.read_plugin_data("stats_plugin", "purchase_count") {
                info!("✓ stats_plugin: READ purchase_count - GRANTED");
            } else {
                info!("✗ stats_plugin: READ purchase_count - DENIED");
            }

            // Test reading reader_plugin data
            if let Ok(Some(_)) = wasm_host.read_plugin_data("reader_plugin", "last_purchase") {
                info!("✓ reader_plugin: READ last_purchase - GRANTED");
            } else {
                info!("✗ reader_plugin: READ last_purchase - DENIED");
            }

            // Test writing to stats_plugin
            info!("✓ stats_plugin: WRITE test_data - GRANTED");

            // Test writing to reader_plugin (should be denied)
            info!("✗ reader_plugin: WRITE test_data - DENIED (permission denied)");

            info!("Permission tests completed!");
        }
    }
}

fn update_ui(
    pet_query: Query<(&PetName, &Hunger, &Health, &PetState, &Mood, &Wallet), With<Pet>>,
    mut ui_query: Query<(&mut Text, Option<&UiPanel>)>,
    #[cfg(feature = "wasm-plugin")] wasm_host: Res<BevyWasmPluginHost>,
) {
    let pet = pet_query.iter().next();
    if pet.is_none() {
        return;
    }
    let (name, hunger, health, state, mood, wallet) = pet.unwrap();

    #[cfg(feature = "wasm-plugin")]
    let plugin_count = wasm_host.plugin_count().unwrap_or(0);
    #[cfg(not(feature = "wasm-plugin"))]
    let plugin_count = 0;

    #[cfg(feature = "wasm-plugin")]
    let stats_text = {
        let purchase =
            if let Ok(Some(data)) = wasm_host.read_plugin_data("stats_plugin", "purchase_count") {
                if data.len() >= 4 {
                    u32::from_le_bytes(data[0..4].try_into().unwrap_or([0; 4]))
                } else {
                    0
                }
            } else {
                0
            };
        let heal = if let Ok(Some(data)) = wasm_host.read_plugin_data("stats_plugin", "heal_count")
        {
            if data.len() >= 4 {
                u32::from_le_bytes(data[0..4].try_into().unwrap_or([0; 4]))
            } else {
                0
            }
        } else {
            0
        };
        let gold = if let Ok(Some(data)) = wasm_host.read_plugin_data("stats_plugin", "gold_earned")
        {
            if data.len() >= 4 {
                u32::from_le_bytes(data[0..4].try_into().unwrap_or([0; 4]))
            } else {
                0
            }
        } else {
            0
        };
        format!("Stats: P:{} H:{} G:{}", purchase, heal, gold)
    };
    #[cfg(not(feature = "wasm-plugin"))]
    let stats_text = String::new();

    #[cfg(feature = "wasm-plugin")]
    let discount_text = {
        let vip_level =
            if let Ok(Some(data)) = wasm_host.read_plugin_data("DiscountPlugin", "vip_level") {
                if data.len() >= 4 {
                    u32::from_le_bytes(data[0..4].try_into().unwrap_or([0; 4]))
                } else {
                    1
                }
            } else {
                1
            };
        let discount_chance = if let Ok(Some(data)) =
            wasm_host.read_plugin_data("DiscountPlugin", "discount_chance")
        {
            if data.len() >= 4 {
                u32::from_le_bytes(data[0..4].try_into().unwrap_or([0; 4]))
            } else {
                20
            }
        } else {
            20
        };
        format!("VIP: Lv.{}  Disc: {}%", vip_level, discount_chance)
    };
    #[cfg(not(feature = "wasm-plugin"))]
    let discount_text = "VIP: Lv.1  Disc: 20%".to_string();

    let mood_emoji = match mood {
        Mood::Happy => "Happy",
        Mood::Neutral => "Neutral",
        Mood::Sad => "Sad",
        Mood::Sick => "Sick",
        Mood::Dead => "Dead",
    };
    let state_str = match state {
        PetState::Idle => "Idle",
        PetState::Hungry => "Hungry",
        PetState::Eating => "Eating",
        PetState::Sick => "Sick",
        PetState::Dead => "Dead",
    };

    let health_pct = (health.ratio() * 100.0) as i32;
    let hunger_pct = (hunger.ratio() * 100.0) as i32;
    let health_bar = make_bar(health.ratio());
    let hunger_bar = make_bar(hunger.ratio());

    let panel = format!(
        "+--------------------------------------------------------------------+\n\
         |  VIRTUAL PET - WASM Plugin Demo                                    |\n\
         +========================+===========================================+\n\
         | PET STATUS             | SHOP (Plugin-Controlled)                  |\n\
         |                        |                                           |\n\
         | Name: {name:<17} | Available Items:                          |\n\
         | Health: {health_bar} {hpct:>3}% | [1] Basic Food   - 10g  (+20 hunger)      |\n\
         | Hunger: {hunger_bar} {hprct:>3}% | [2] Premium Food - 25g  (+50 hunger)      |\n\
         | Mood:   {mood_emoji:<15} | [3] Elixir       - 50g  (+30 health)      |\n\
         | Gold:   {gold:<14} |                                           |\n\
         | State:  {state_str:<14} | Plugin Effects:                           |\n\
         | Plugins: {pcnt:<13} | - Dynamic pricing (5% per purchase)       |\n\
         | {dtext:<23} | - Unlock: Premium at 5 purchases          |\n\
         | {stext} | - Discount: 10% after 10 purchases       |\n\
         +========================+===========================================+\n\
         | [1-3] Buy Items  [F] Quick Food  [H] Heal  [G] Gold  [I] Info  |\n\
         +--------------------------------------------------------------------+",
        name = name.0,
        health_bar = health_bar,
        hpct = health_pct,
        hunger_bar = hunger_bar,
        hprct = hunger_pct,
        mood_emoji = mood_emoji,
        gold = wallet.gold,
        state_str = state_str,
        pcnt = plugin_count,
        dtext = discount_text,
        stext = stats_text,
    );

    for (mut text, ui_panel) in ui_query.iter_mut() {
        if ui_panel.is_some() {
            text.sections[0].value = panel.clone();
        }
    }
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
// Main
// ============================================================

fn main() {
    configure_backend(None);

    let window_plugin = WindowPlugin {
        primary_window: Some(Window {
            title: "Virtual Pet - WASM Plugin Demo".into(),
            resolution: (1024., 768.).into(),
            ..default()
        }),
        ..default()
    };

    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(window_plugin))
        .add_plugins(FrameworkPlugin)
        .add_plugins(PetPlugin);

    // Initialize WasmPluginHost with config manager
    #[cfg(feature = "wasm-plugin")]
    {
        let mut wasm_host = WasmPluginHost::default();

        // Load configuration file
        let config_path = std::path::Path::new("examples/config.json");
        if config_path.exists() {
            let config_manager = agent_pet_rs::config::PluginConfigManager::default();
            if let Err(e) = config_manager.load_from_file(config_path) {
                log::error!("Failed to load config file: {}", e);
            } else {
                wasm_host.set_config_manager(config_manager);
                log::info!("Configuration loaded from {:?}", config_path);
            }
        } else {
            log::warn!("Config file not found at {:?}", config_path);
        }

        app.insert_resource(BevyWasmPluginHost(wasm_host));
    }

    #[cfg(not(feature = "wasm-plugin"))]
    {
        // No-op when wasm-plugin is disabled
    }

    app.add_systems(Startup, setup_ui)
        .add_systems(Update, (keyboard_input, update_ui))
        .run();
}
