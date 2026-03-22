//! WASM plugin runtime using wasmtime.
//!
//! ## Design Decision: Session-per-call
//!
//! Wasmtime requires `&mut Store` to call functions, but `WasmPlugin::on_tick(&self)`
//! only has immutable access. We solve this by storing the wasm bytes and
//! re-instantiating a new Store for each call. This is simple and works for
//! the example use case. For production, you'd refactor `WasmPluginHost` to
//! own the Store.
//!
//! ## String Transfer Protocol
//!
//! Host allocates a scratch buffer (64KB) and passes (ptr, len) to wasm.
//! Wasm writes string bytes directly to linear memory, no copy needed.

#[cfg(feature = "wasm-plugin")]
use {
    crate::error::FrameworkError,
    crate::wasm::{WasmEntityId, WasmPlugin, WasmPluginId},
    std::fs,
    wasmtime::{Engine, Instance, Linker, Memory, Module, Store, TypedFunc},
};

/// WASM plugin instance wrapper (store-per-call strategy).
#[cfg(feature = "wasm-plugin")]
pub struct WasmtimePlugin {
    id: WasmPluginId,
    name: String,
    wasm_bytes: Vec<u8>,
    engine: Engine,
}

#[cfg(feature = "wasm-plugin")]
impl WasmtimePlugin {
    const SCRATCH_SIZE: usize = 64 * 1024; // 64KB scratch buffer

    /// Load a WASM plugin from file path.
    ///
    /// ## ABI
    ///
    /// The wasm file must export:
    /// - `fn wasm_plugin_name() -> u32` (pointer to name in memory)
    /// - `fn wasm_plugin_name_len() -> u32` (length of name)
    /// - `fn wasm_plugin_on_tick(entity_id: u64)`
    /// - `fn wasm_plugin_on_event(entity_id: u64, event_ptr: u32, event_len: u32, data_ptr: u32, data_len: u32)`
    ///
    /// ## Memory Layout
    ///
    /// Wasm memory is expected to have at least one page (64KB).
    /// Strings are written into the scratch region by wasm.
    pub fn load(
        wasm_path: &std::path::Path,
        override_id: Option<String>,
    ) -> Result<Self, FrameworkError> {
        let wasm_bytes = fs::read(wasm_path)
            .map_err(|e| FrameworkError::WasmLoad(format!("failed to read file: {}", e)))?;

        let engine = Engine::default();

        // Create a temporary instance just to read the name
        let (name, _instance, store, _memory, _on_tick, _on_event) =
            Self::create_instance_with_name(&engine, &wasm_bytes)?;
        let id = WasmPluginId::new(override_id.unwrap_or_else(|| name.clone()));

        // Drop the temporary store
        let _ = store;

        Ok(Self {
            id,
            name,
            wasm_bytes,
            engine,
        })
    }

    /// Create an instance (engine + store + instance) for a single call.
    fn create_instance(
        engine: &Engine,
        wasm_bytes: &[u8],
    ) -> Result<
        (
            String,
            Instance,
            Store<()>,
            Memory,
            TypedFunc<u64, ()>,
            TypedFunc<(u64, u32, u32, u32, u32), ()>,
        ),
        FrameworkError,
    > {
        let module = Module::new(engine, wasm_bytes)
            .map_err(|e| FrameworkError::WasmLoad(format!("compile failed: {}", e)))?;

        let mut linker = Linker::new(engine);
        linker
            .define_unknown_imports_as_traps(&module)
            .map_err(|e| FrameworkError::WasmLoad(format!("link failed: {}", e)))?;

        let mut store = Store::new(engine, ());
        let instance = linker
            .instantiate(&mut store, &module)
            .map_err(|e| FrameworkError::WasmLoad(format!("instantiate failed: {}", e)))?;

        let memory = instance
            .get_memory(&mut store, "memory")
            .ok_or_else(|| FrameworkError::WasmLoad("memory not found".into()))?;

        // Ensure minimum memory size (1 page = 64KB)
        let current_pages = memory.size(&store);
        if current_pages < 1 {
            memory
                .grow(&mut store, 1)
                .map_err(|e| FrameworkError::WasmLoad(format!("grow memory failed: {}", e)))?;
        }

        let on_tick = instance
            .get_typed_func::<u64, ()>(&mut store, "wasm_plugin_on_tick")
            .map_err(|e| FrameworkError::WasmLoad(format!("missing wasm_plugin_on_tick: {}", e)))?;

        let on_event = instance
            .get_typed_func::<(u64, u32, u32, u32, u32), ()>(&mut store, "wasm_plugin_on_event")
            .map_err(|e| {
                FrameworkError::WasmLoad(format!("missing wasm_plugin_on_event: {}", e))
            })?;

        Ok((String::new(), instance, store, memory, on_tick, on_event))
    }

    /// Create instance and read name in one go.
    fn create_instance_with_name(
        engine: &Engine,
        wasm_bytes: &[u8],
    ) -> Result<
        (
            String,
            Instance,
            Store<()>,
            Memory,
            TypedFunc<u64, ()>,
            TypedFunc<(u64, u32, u32, u32, u32), ()>,
        ),
        FrameworkError,
    > {
        let (_, instance, mut store, memory, on_tick, on_event) =
            Self::create_instance(engine, wasm_bytes)?;

        // Read plugin name
        let name_ptr = instance
            .get_typed_func::<(), u32>(&mut store, "wasm_plugin_name")
            .map_err(|e| FrameworkError::WasmLoad(format!("missing wasm_plugin_name: {}", e)))?
            .call(&mut store, ())
            .map_err(|e| {
                FrameworkError::WasmLoad(format!("wasm_plugin_name call failed: {}", e))
            })?;

        let name_len = instance
            .get_typed_func::<(), u32>(&mut store, "wasm_plugin_name_len")
            .map_err(|e| FrameworkError::WasmLoad(format!("missing wasm_plugin_name_len: {}", e)))?
            .call(&mut store, ())
            .map_err(|e| {
                FrameworkError::WasmLoad(format!("wasm_plugin_name_len call failed: {}", e))
            })?;

        // Read from wasm memory
        let data = memory.data(&store);
        let start = name_ptr as usize;
        let end = start + name_len as usize;

        if end > data.len() {
            return Err(FrameworkError::WasmLoad(
                "plugin name pointer out of bounds".into(),
            ));
        }

        let name_bytes = &data[start..end];
        let name = String::from_utf8(name_bytes.to_vec())
            .map_err(|e| FrameworkError::WasmLoad(format!("plugin name not valid UTF-8: {}", e)))?;

        Ok((name, instance, store, memory, on_tick, on_event))
    }
}

#[cfg(feature = "wasm-plugin")]
impl WasmPlugin for WasmtimePlugin {
    fn id(&self) -> &WasmPluginId {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn on_tick(&self, entity_id: WasmEntityId) {
        // Create a new instance for this call
        let result = Self::create_instance_with_name(&self.engine, &self.wasm_bytes);
        match result {
            Ok((_name, _instance, mut store, _memory, on_tick, _on_event)) => {
                // Call the wasm function
                let _ = on_tick.call(&mut store, entity_id.0);
            }
            Err(e) => {
                log::error!("WASM on_tick failed: {}", e);
            }
        }
    }

    fn on_event(&self, entity_id: WasmEntityId, event: &str, data: &str) {
        // Create a new instance for this call
        let result = Self::create_instance_with_name(&self.engine, &self.wasm_bytes);
        match result {
            Ok((_name, _instance, mut store, memory, _on_tick, on_event)) => {
                // Write strings to wasm memory scratch region
                let scratch_offset = 0;
                let event_bytes = event.as_bytes();
                let data_bytes = data.as_bytes();

                // Get memory slice
                let mem_data = memory.data_mut(&mut store);
                let scratch_start = scratch_offset;
                let scratch_end = scratch_start + Self::SCRATCH_SIZE;

                if scratch_end > mem_data.len() {
                    log::error!("WASM memory too small for scratch buffer");
                    return;
                }

                // Write event
                let event_ptr = scratch_start as u32;
                let event_len = event_bytes.len() as u32;
                mem_data[scratch_start..scratch_start + event_bytes.len()]
                    .copy_from_slice(event_bytes);

                // Write data after event
                let data_ptr = (scratch_start + event_bytes.len()) as u32;
                let data_len = data_bytes.len() as u32;
                let data_ptr_usize = data_ptr as usize;
                if data_ptr_usize + data_bytes.len() > scratch_end {
                    log::error!("WASM scratch buffer overflow");
                    return;
                }
                mem_data[data_ptr_usize..data_ptr_usize + data_bytes.len()]
                    .copy_from_slice(data_bytes);

                // Call the wasm function
                let _ = on_event.call(
                    &mut store,
                    (entity_id.0, event_ptr, event_len, data_ptr, data_len),
                );
            }
            Err(e) => {
                log::error!("WASM on_event failed: {}", e);
            }
        }
    }
}
