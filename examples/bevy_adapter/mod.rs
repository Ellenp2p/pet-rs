//! Bevy adapter for agent-pet-rs framework.
//!
//! This module provides Bevy-specific wrappers for the core agent-pet-rs types.
//! It should be used in examples that use Bevy for rendering.

use agent_pet_rs::prelude::*;
use bevy::prelude::*;

// ============================================================
// Bevy Resource Wrappers
// ============================================================

/// Bevy wrapper for HookRegistry.
#[derive(Resource, Default)]
pub struct BevyHookRegistry(pub HookRegistry);

impl BevyHookRegistry {
    pub fn trigger(
        &self,
        key: &str,
        ctx: &agent_pet_rs::hooks::HookContext,
    ) -> Result<(), FrameworkError> {
        self.0.trigger(key, ctx)?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn register(
        &mut self,
        hook_point: HookPoint,
        priority: i32,
        callback: agent_pet_rs::hooks::registry::HookCallback,
    ) -> Result<(), FrameworkError> {
        self.0.register(hook_point, priority, callback)
    }
}

/// Bevy wrapper for NetworkConfig.
#[derive(Resource, Deref, DerefMut, Clone, Default)]
pub struct BevyNetworkConfig(pub NetworkConfig);

/// Bevy wrapper for PluginConfigManager.
#[derive(Resource, Deref, DerefMut, Default)]
pub struct BevyPluginConfigManager(pub PluginConfigManager);

/// Bevy wrapper for WasmPluginHost.
#[cfg(feature = "wasm-plugin")]
#[derive(Resource, Deref, DerefMut)]
pub struct BevyWasmPluginHost(pub WasmPluginHost);

#[cfg(feature = "wasm-plugin")]
impl Default for BevyWasmPluginHost {
    fn default() -> Self {
        Self(WasmPluginHost::default())
    }
}

// ============================================================
// Framework Plugin
// ============================================================

/// Framework-level system sets for pipeline ordering.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum FrameworkSet {
    Input,
    Process,
    Output,
}

/// The core framework plugin for Bevy.
///
/// Registers `HookRegistry`, `NetworkConfig`, `PluginConfigManager`, and `FrameworkSet` ordering.
pub struct FrameworkPlugin;

impl Plugin for FrameworkPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BevyHookRegistry>()
            .init_resource::<BevyNetworkConfig>()
            .init_resource::<BevyPluginConfigManager>()
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

/// WASM plugin support for Bevy.
#[cfg(feature = "wasm-plugin")]
pub struct WasmPluginBevy;

#[cfg(feature = "wasm-plugin")]
impl Plugin for WasmPluginBevy {
    fn build(&self, app: &mut App) {
        app.init_resource::<BevyWasmPluginHost>();
    }
}

// ============================================================
// Helper Functions
// ============================================================

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
