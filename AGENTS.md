# AGENTS.md

Instructions for AI agents working on this codebase.

## Build & Verify Commands

Run these before any commit. ALL must pass:

```bash
cargo check                    # compile library
cargo check --example basic_pet --features wasm-plugin  # compile example with plugins
cargo test                     # run tests (28 tests)
cargo clippy                   # lint
cargo fmt --check              # formatting
```

Fix formatting with: `cargo fmt`

## Project Structure

- `src/` — Generic framework library only. NO domain-specific code (no pets, no economy).
- `examples/basic_pet.rs` — Bevy GUI demo with WASM plugins.
- `examples/cli_pet.rs` — CLI/TUI demo.
- `tests/pet_tests.rs` — Tests for framework generics (HookRegistry, NetworkChannel).

## Key Design Rules

1. **Framework is domain-agnostic.** `src/` must never reference "pet", "hunger", "health", "wallet", etc.
2. **All mutations go through events.** No direct component mutation outside systems.
3. **HookRegistry uses `Cow<'static, str>` keys.** Plugins define their own hook constants.
4. **NetworkChannel\<T\> is generic.** No hardcoded DTO types in framework.
5. **No global mutable state.** Use Bevy Resources or safe patterns.

## Adding New Features

To add a new feature to the agent demo (not the framework):

1. Add component/event in `examples/basic_pet.rs`
2. Register in `PetPlugin::build()`
3. Write systems, assign to `PetSet` or `FrameworkSet`

To add framework infrastructure:

1. Add generic code in `src/`
2. Export from `src/prelude.rs`
3. Add tests in `tests/pet_tests.rs`
4. Ensure zero domain-specific names

## Dependencies

- Bevy 0.14 (full)
- tokio, reqwest, tokio-tungstenite (networking)
- wasmtime 24 (optional, feature `wasm-plugin`)
- serde, serde_json (serialization)
- ratatui, crossterm (CLI/TUI)

## Backend

Default rendering backend is Vulkan. Set via `configure_backend()` before `App::new()`.
Override with env var: `WGPU_BACKEND=opengl` or `WGPU_BACKEND=dx12`.

## CI

`.github/workflows/ci.yml` — runs fmt, check, test on ubuntu-latest with Rust stable.
