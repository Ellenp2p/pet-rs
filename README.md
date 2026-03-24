# agent-pet-rs

Rust framework for building intelligent virtual life forms with WASM plugins.

## Philosophy

The framework provides **infrastructure for creating customizable intelligent agents**. Domain logic (behaviors, interactions, etc.) lives entirely in user code via the plugin system.

| Layer | Location | What it contains |
|-------|----------|------------------|
| Framework | `src/` | HookRegistry, NetworkChannel, WASM bridge, Permission management |
| Plugin | `examples/` | Agent components, events, systems — built on framework API |

## Quick Start

```bash
# Run the agent demo (Bevy GUI)
cargo run --example basic_pet --features wasm-plugin

# Run the agent demo (CLI/TUI)
cargo run --example cli_pet

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
| F | Feed agent (costs 10 gold, +20 hunger) |
| H | Heal agent (+15 health) |
| G | Gain gold (+50) |
| 1 | Buy Basic Food (10g) |
| 2 | Buy Premium Food (25g) |
| 3 | Buy Elixir (50g) |
| R | Hot reload plugins |
| I | Show plugin info |
| P | Test permissions |

## Architecture

```
FrameworkSet::Input → Plugin systems (spawn, input)
FrameworkSet::Process → Plugin systems (simulation, economy)
FrameworkSet::Output → Plugin systems (UI, network sync)
```

Plugins extend the pipeline by inserting their own `SystemSet` between framework stages.

## Project Structure

```
agent-pet-rs/
├── src/                          # Framework core (generic, no domain knowledge)
│   ├── lib.rs                    # FrameworkPlugin, FrameworkSet
│   ├── prelude.rs                # Public API exports
│   ├── hooks/
│   │   └── hook_system.rs        # HookRegistry with Cow<str> keys
│   ├── network/
│   │   └── mod.rs                # NetworkChannel<T> generic channel
│   ├── config.rs                 # Plugin configuration
│   ├── dependency.rs             # Dependency resolution
│   ├── permission.rs             # Permission management
│   └── wasm/
│       ├── plugin_trait.rs       # WasmPlugin trait
│       ├── wasmtime_loader.rs    # WASM plugin loader
│       └── bridge.rs             # WasmPluginHost
├── examples/
│   ├── basic_pet.rs              # Bevy GUI demo
│   ├── cli_pet.rs                # CLI/TUI demo
│   ├── bevy_adapter.rs           # Bevy integration adapter
│   ├── wasm_hooks/               # Demo WASM plugin
│   ├── wasm_stats/               # Stats WASM plugin
│   ├── wasm_reader/              # Reader WASM plugin
│   ├── wasm_discount/            # Discount WASM plugin
│   └── config.json               # Plugin configuration
├── tests/
│   └── agent_tests.rs            # Framework unit tests
└── .github/workflows/ci.yml     # CI: fmt + check + test
```

## Core API

### HookRegistry

Dynamic, string-keyed event hooks with multiple subscribers:

```rust
use agent_pet_rs::prelude::*;

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
use agent_pet_rs::prelude::*;

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
