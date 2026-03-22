#![allow(clippy::type_complexity)]

use bevy::prelude::*;
use pet_rs::prelude::*;
use std::collections::HashMap;

#[cfg(not(feature = "wasm-plugin"))]
mod wasm_stub {
    use bevy::prelude::Resource;
    use pet_rs::error::FrameworkError;
    #[derive(Default, Resource)]
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
    }
}
#[cfg(not(feature = "wasm-plugin"))]
use wasm_stub::WasmPluginHost;

// ============================================================
// Components
// ============================================================

#[derive(Component, Debug, Clone)]
struct Pet;

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
    hooks: Res<HookRegistry>,
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

        hooks.trigger(ON_SPAWN, &HookContext { entity });
        info!("Spawned pet '{}'", ev.name);
    }
}

fn feed_input_system(
    mut events: EventReader<FeedEvent>,
    mut pet_query: Query<(&mut Hunger, Option<&Wallet>), With<Pet>>,
    hooks: Res<HookRegistry>,
) {
    for ev in events.read() {
        if let Ok((mut hunger, _wallet)) = pet_query.get_mut(ev.entity) {
            hunger.value = (hunger.value + ev.amount).clamp(0.0, hunger.max);
            hooks.trigger(ON_FEED, &HookContext { entity: ev.entity });
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
    hooks: Res<HookRegistry>,
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

        hooks.trigger(ON_PURCHASE, &HookContext { entity: ev.entity });
        info!("Purchase '{}' by {:?}", ev.item, ev.entity);
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
    hooks: Res<HookRegistry>,
) {
    for ev in events.read() {
        if let Ok(mut wallet) = pet_query.get_mut(ev.entity) {
            if ev.currency == "gold" {
                wallet.gold += ev.amount;
                hooks.trigger(ON_REWARD, &HookContext { entity: ev.entity });
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
    _wasm_host: Res<WasmPluginHost>,
) {
    commands.spawn(Camera2dBundle::default());

    spawn_events.send(SpawnPetEvent {
        name: "Buddy".into(),
    });

    // Load WASM plugin if feature enabled
    #[cfg(feature = "wasm-plugin")]
    {
        use std::path::Path;
        let wasm_path =
            Path::new("examples/wasm_hooks/target/wasm32-unknown-unknown/release/wasm_hooks.wasm");
        if wasm_path.exists() {
            match _wasm_host.register_wasm(wasm_path, Some("demo_plugin".into())) {
                Ok(()) => {
                    info!("WASM plugin loaded successfully");
                    let count = _wasm_host.plugin_count().unwrap_or(0);
                    info!("Total WASM plugins: {}", count);
                }
                Err(e) => error!("Failed to load WASM plugin: {}", e),
            }
        } else {
            warn!("WASM plugin file not found at {:?}", wasm_path);
        }
    }

    commands.spawn(
        TextBundle::from_section(
            "Loading...",
            TextStyle {
                font_size: 28.0,
                color: Color::WHITE,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(30.0),
            left: Val::Px(30.0),
            ..default()
        }),
    );
}

fn keyboard_input(
    keys: Res<ButtonInput<KeyCode>>,
    pet_query: Query<Entity, With<Pet>>,
    mut purchase_events: EventWriter<PurchaseEvent>,
    mut heal_events: EventWriter<HealEvent>,
    mut gain_events: EventWriter<GainEvent>,
    _wasm_host: Res<WasmPluginHost>,
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
            let _ = _wasm_host.trigger_on_event(entity.index() as u64, "purchase", "food");
        }
        if keys.just_pressed(KeyCode::KeyH) {
            heal_events.send(HealEvent {
                entity,
                amount: 15.0,
            });
            #[cfg(feature = "wasm-plugin")]
            let _ = _wasm_host.trigger_on_event(entity.index() as u64, "heal", "15");
        }
        if keys.just_pressed(KeyCode::KeyG) {
            gain_events.send(GainEvent {
                entity,
                currency: "gold".into(),
                amount: 50,
            });
            #[cfg(feature = "wasm-plugin")]
            let _ = _wasm_host.trigger_on_event(entity.index() as u64, "gain_gold", "50");
        }
    }
}

fn update_ui(
    pet_query: Query<(&PetName, &Hunger, &Health, &PetState, &Mood, &Wallet), With<Pet>>,
    mut text_query: Query<&mut Text>,
    _wasm_host: Res<WasmPluginHost>,
) {
    for (name, hunger, health, state, mood, wallet) in pet_query.iter() {
        for mut text in text_query.iter_mut() {
            #[cfg(feature = "wasm-plugin")]
            let plugin_count = _wasm_host.plugin_count().unwrap_or(0);
            #[cfg(not(feature = "wasm-plugin"))]
            let plugin_count = 0;

            let lines = vec![
                "===========================".into(),
                String::new(),
                format!("      {}", name.0),
                String::new(),
                "===========================".into(),
                String::new(),
                format!("  Hunger: {:.0} / {:.0}", hunger.value, hunger.max),
                format!("  Health: {:.0} / {:.0}", health.value, health.max),
                format!("  State:  {:?}", state),
                format!("  Mood:   {:?}", mood),
                format!("  Gold:   {}", wallet.gold),
                format!("  WASM Plugins: {}", plugin_count),
                String::new(),
                "  [F] Buy Food (-10g)".into(),
                "  [H] Heal".into(),
                "  [G] Gain Gold (+50g)".into(),
                "===========================".into(),
            ];
            text.sections[0].value = lines.join("\n");
        }
    }
}

// ============================================================
// Main
// ============================================================

fn main() {
    configure_backend(None);
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(FrameworkPlugin)
        .add_plugins(PetPlugin)
        .insert_resource(WasmPluginHost::default())
        .add_systems(Startup, setup_ui)
        .add_systems(Update, (keyboard_input, update_ui))
        .run();
}
