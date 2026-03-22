use bevy::prelude::*;

pub struct WasmPlugin;

impl Plugin for WasmPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<crate::wasm::WasmPluginHost>();
    }
}
