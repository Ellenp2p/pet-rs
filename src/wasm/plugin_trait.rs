/// Opaque entity identifier for use across the WASM boundary.
///
/// WASM plugins cannot hold Bevy `Entity` references directly.
/// This newtype wraps the raw `u64` representation with type safety.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WasmEntityId(pub u64);

/// Unique identifier for a WASM plugin instance.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WasmPluginId(String);

impl WasmPluginId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

pub trait WasmPlugin: Send + Sync {
    fn id(&self) -> &WasmPluginId;
    fn name(&self) -> &str;
    fn on_tick(&self, entity_id: WasmEntityId);
    fn on_event(&self, entity_id: WasmEntityId, event: &str, data: &str);
}
