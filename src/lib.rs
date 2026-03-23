#![allow(clippy::type_complexity)]

pub mod config;
pub mod dependency;
pub mod error;
pub mod hooks;
pub mod network;
pub mod permission;
pub mod plugins;

pub mod components;
pub mod events;
pub mod systems;

#[cfg(feature = "wasm-plugin")]
pub mod wasm;

pub mod prelude;

use bevy::prelude::*;

use hooks::HookRegistry;

/// Framework-level system sets for pipeline ordering.
///
/// Plugins insert their own `SystemSet` between these stages:
/// ```ignore
/// FrameworkSet::Input → MyPluginSet → FrameworkSet::Process → FrameworkSet::Output
/// ```
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum FrameworkSet {
    Input,
    Process,
    Output,
}

/// Convenience function to set the WGPU rendering backend.
///
/// If `WGPU_BACKEND` is not already set, defaults to `"vulkan"`.
///
/// **Warning:** This sets a process-wide environment variable. Do not call
/// concurrently from multiple threads. Prefer calling once at the start
/// of `main()` before `App::new()`.
pub fn configure_backend(backend: Option<&str>) {
    if let Some(b) = backend {
        std::env::set_var("WGPU_BACKEND", b);
    } else if std::env::var("WGPU_BACKEND").is_err() {
        std::env::set_var("WGPU_BACKEND", "vulkan");
    }
}

/// The core framework plugin.
///
/// Registers `HookRegistry`, `NetworkConfig`, `PluginConfigManager`, and `FrameworkSet` ordering.
pub struct FrameworkPlugin;

impl Plugin for FrameworkPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HookRegistry>()
            .init_resource::<network::NetworkConfig>()
            .init_resource::<config::PluginConfigManager>()
            .configure_sets(
                Update,
                (
                    FrameworkSet::Input,
                    FrameworkSet::Process,
                    FrameworkSet::Output,
                )
                    .chain(),
            );
    }
}
