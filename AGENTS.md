# AGENTS.md

Instructions for AI agents working on this codebase.

## Build & Verify Commands

Run these before any commit. ALL must pass:

```bash
cargo check                    # compile library
cargo check --example basic_pet  # compile example
cargo test                     # run tests (11 tests)
cargo clippy                   # lint
cargo clippy --example basic_pet
cargo fmt --check              # formatting
```

Fix formatting with: `cargo fmt`

## Project Structure

- `src/` — Generic framework library only. NO domain-specific code (no pets, no economy).
- `examples/basic_pet.rs` — Self-contained demo. ALL pet/economy components, events, systems, and UI live here.
- `tests/pet_tests.rs` — Tests for framework generics (HookRegistry, NetworkChannel).

## Key Design Rules

1. **Framework is domain-agnostic.** `src/` must never reference "pet", "hunger", "health", "wallet", etc.
2. **All mutations go through events.** No direct component mutation outside systems.
3. **System ordering uses FrameworkSet.** Plugins extend the pipeline, don't replace it.
4. **HookRegistry uses `Cow<'static, str>` keys.** Plugins define their own hook constants.
5. **NetworkChannel\<T\> is generic.** No hardcoded DTO types in framework.
6. **No global mutable state.** Use Bevy Resources.

## Adding New Features

To add a new feature to the pet demo (not the framework):

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

## Backend

Default rendering backend is Vulkan. Set via `configure_backend()` before `App::new()`.
Override with env var: `WGPU_BACKEND=opengl` or `WGPU_BACKEND=dx12`.

## CI

`.github/workflows/ci.yml` — runs fmt, check, test on ubuntu-latest with Rust stable.
