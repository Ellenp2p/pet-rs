#![allow(clippy::type_complexity)]

pub mod components;
pub mod events;
pub mod hooks;
pub mod plugins;
pub mod systems;
pub mod network;

#[cfg(feature = "wasm-plugin")]
pub mod wasm;

pub mod prelude;

use bevy::prelude::*;

use plugins::pet_plugin::PetPlugin;
use plugins::network_plugin::NetworkPlugin;

pub fn configure_backend() {
    if std::env::var("WGPU_BACKEND").is_err() {
        std::env::set_var("WGPU_BACKEND", "vulkan");
    }
}

pub struct PetFrameworkPlugin;

impl Plugin for PetFrameworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PetPlugin)
            .add_plugins(NetworkPlugin);
    }
}
