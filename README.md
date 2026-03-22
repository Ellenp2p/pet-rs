# pet-rs

ECS-first plugin framework built on Bevy, with hook system, generic networking, and WASM plugin support.

## Philosophy

The framework provides **only infrastructure**. Domain logic (pets, economy, etc.) lives entirely in user code via the plugin system.

| Layer | Location | What it contains |
|-------|----------|------------------|
| Framework | `src/` | HookRegistry, NetworkChannel, FrameworkSet, WASM bridge |
| Plugin | `examples/` | Pet components, events, systems, economy — built on framework API |

## Quick Start

```bash
# Run the pet demo
cargo run --example basic_pet

# Build the framework library only
cargo build

# Run tests
cargo test

# Check formatting + lint
cargo fmt --check
cargo clippy
```

**Controls (in demo):**

| Key | Action |
|-----|--------|
| F | Buy food (costs 10 gold, feeds pet +20 hunger) |
| H | Heal pet (+15 health) |
| G | Gain gold (+50) |

## Architecture

```
FrameworkSet::Input → Plugin systems (spawn, input)
FrameworkSet::Process → Plugin systems (simulation, economy)
FrameworkSet::Output → Plugin systems (UI, network sync)
```

Plugins extend the pipeline by inserting their own `SystemSet` between framework stages.

## Project Structure

```
pet-rs/
├── src/                          # Framework core (generic, no domain knowledge)
│   ├── lib.rs                    # FrameworkPlugin, FrameworkSet
│   ├── prelude.rs                # Public API exports
│   ├── hooks/
│   │   └── hook_system.rs        # HookRegistry with Cow<str> keys
│   ├── network/
│   │   └── mod.rs                # NetworkChannel<T> generic channel
│   ├── plugins/
│   │   └── wasm_plugin.rs        # WASM plugin Bevy integration
│   └── wasm/
│       ├── plugin_trait.rs       # WasmPlugin trait
│       └── bridge.rs             # WasmPluginHost
├── examples/
│   └── basic_pet.rs              # Pet + economy demo (self-contained plugin)
├── tests/
│   └── pet_tests.rs              # Framework unit tests
└── .github/workflows/ci.yml     # CI: fmt + check + test
```

## Core API

### HookRegistry

Dynamic, string-keyed event hooks with multiple subscribers:

```rust
use pet_rs::prelude::*;

// Register
hooks.register_fn("my_hook", |ctx: &HookContext| {
    println!("triggered on {:?}", ctx.entity);
});

// Trigger
hooks.trigger("my_hook", &HookContext { entity });
```

### NetworkChannel\<T\>

Generic bidirectional channel for async networking:

```rust
let channel: NetworkChannel<MyDto> = NetworkChannel::default();

// Send outgoing
channel.send(dto)?;

// Receive incoming
let msgs = channel.drain_incoming();
```

### FrameworkSet

Ordered system sets for pipeline control:

```rust
app.configure_sets(Update, (
    FrameworkSet::Input,
    MyCustomSet,              // plugin inserts here
    FrameworkSet::Process,
    FrameworkSet::Output,
)).chain();
```

### WasmPlugin

Trait for WASM-based dynamic plugins (feature-gated):

```rust
// cargo build --features wasm-plugin
impl WasmPlugin for MyPlugin {
    fn name(&self) -> &str { "my_plugin" }
    fn on_tick(&self, entity_id: u64) { }
    fn on_event(&self, entity_id: u64, event: &str, data: &str) { }
}
```

## Building a Plugin

See `examples/basic_pet.rs` for a complete example. Minimal structure:

```rust
use bevy::prelude::*;
use pet_rs::prelude::*;

struct MyPlugin;

impl Plugin for MyPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MyEvent>()
            .add_systems(Update, my_system.in_set(FrameworkSet::Process));
    }
}

fn main() {
    configure_backend();
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(FrameworkPlugin)
        .add_plugins(MyPlugin)
        .run();
}
```

## CI

GitHub Actions runs on every push/PR to `master`:

- `cargo fmt --check`
- `cargo check --all-targets`
- `cargo test`

## License

TBD
