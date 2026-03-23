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
    std::sync::Arc,
    wasmtime::{Caller, Engine, Instance, Linker, Memory, Module, Store, TypedFunc},
};

/// Data stored in wasmtime Store for host function access.
#[cfg(feature = "wasm-plugin")]
pub struct StoreData {
    /// Plugin ID for this instance
    pub plugin_id: WasmPluginId,
    /// Callback to read plugin data
    pub read_data_fn: Option<Arc<dyn Fn(&str, &str) -> Option<Vec<u8>> + Send + Sync>>,
    /// Callback to write plugin data
    pub write_data_fn: Option<Arc<dyn Fn(&str, &str, Vec<u8>) + Send + Sync>>,
}

/// WASM plugin instance wrapper (store-per-call strategy).
#[cfg(feature = "wasm-plugin")]
pub struct WasmtimePlugin {
    id: WasmPluginId,
    name: String,
    wasm_bytes: Vec<u8>,
    engine: Engine,
    state: std::sync::RwLock<Vec<u8>>,
    /// Callback to read plugin data (for inter-plugin communication)
    read_data_fn: Option<Arc<dyn Fn(&str, &str) -> Option<Vec<u8>> + Send + Sync>>,
    /// Callback to write plugin data (for inter-plugin communication)
    write_data_fn: Option<Arc<dyn Fn(&str, &str, Vec<u8>) + Send + Sync>>,
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
            state: std::sync::RwLock::new(Vec::new()),
            read_data_fn: None,
            write_data_fn: None,
        })
    }

    /// Load a WASM plugin with data callbacks for inter-plugin communication.
    pub fn load_with_callbacks(
        wasm_path: &std::path::Path,
        override_id: Option<String>,
        read_data_fn: Arc<dyn Fn(&str, &str) -> Option<Vec<u8>> + Send + Sync>,
        write_data_fn: Arc<dyn Fn(&str, &str, Vec<u8>) + Send + Sync>,
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
            state: std::sync::RwLock::new(Vec::new()),
            read_data_fn: Some(read_data_fn),
            write_data_fn: Some(write_data_fn),
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

    /// Create instance with custom host functions for data read/write.
    fn create_instance_with_data_callbacks(
        engine: &Engine,
        wasm_bytes: &[u8],
        plugin_id: WasmPluginId,
        read_data_fn: Arc<dyn Fn(&str, &str) -> Option<Vec<u8>> + Send + Sync>,
        write_data_fn: Arc<dyn Fn(&str, &str, Vec<u8>) + Send + Sync>,
    ) -> Result<
        (
            Instance,
            Store<StoreData>,
            Memory,
            TypedFunc<u64, ()>,
            TypedFunc<(u64, u32, u32, u32, u32), ()>,
        ),
        FrameworkError,
    > {
        let module = Module::new(engine, wasm_bytes)
            .map_err(|e| FrameworkError::WasmLoad(format!("compile failed: {}", e)))?;

        let mut linker = Linker::<StoreData>::new(engine);

        // Define host function: wasm_plugin_set_data(key_ptr, key_len, data_ptr, data_len)
        let read_data_fn_clone = read_data_fn.clone();
        let write_data_fn_clone = write_data_fn.clone();

        linker
            .func_wrap(
                "env",
                "wasm_plugin_set_data",
                move |mut caller: Caller<'_, StoreData>,
                      key_ptr: u32,
                      key_len: u32,
                      data_ptr: u32,
                      data_len: u32| {
                    let memory = caller
                        .get_export("memory")
                        .and_then(|e| e.into_memory())
                        .unwrap();
                    let mem_data = memory.data(&caller);

                    // Read key
                    let key_start = key_ptr as usize;
                    let key_end = key_start + key_len as usize;
                    if key_end > mem_data.len() {
                        return;
                    }
                    let key_bytes = &mem_data[key_start..key_end];
                    let key = core::str::from_utf8(key_bytes).unwrap_or("");

                    // Read data
                    let data_start = data_ptr as usize;
                    let data_end = data_start + data_len as usize;
                    if data_end > mem_data.len() {
                        return;
                    }
                    let data = mem_data[data_start..data_end].to_vec();

                    // Write data
                    write_data_fn_clone(key, key, data);
                },
            )
            .map_err(|e| {
                FrameworkError::WasmLoad(format!("failed to define wasm_plugin_set_data: {}", e))
            })?;

        linker
            .func_wrap(
                "env",
                "wasm_plugin_read_data",
                move |mut caller: Caller<'_, StoreData>,
                      key_ptr: u32,
                      key_len: u32,
                      result_ptr: u32,
                      result_max_len: u32|
                      -> u32 {
                    let memory = caller
                        .get_export("memory")
                        .and_then(|e| e.into_memory())
                        .unwrap();
                    let mem_data = memory.data(&caller);

                    // Read key
                    let key_start = key_ptr as usize;
                    let key_end = key_start + key_len as usize;
                    if key_end > mem_data.len() {
                        return 0;
                    }
                    let key_bytes = &mem_data[key_start..key_end];
                    let key = core::str::from_utf8(key_bytes).unwrap_or("");

                    // Read data from host
                    if let Some(data) = read_data_fn_clone(key, key) {
                        let result_start = result_ptr as usize;
                        let result_end = result_start + data.len().min(result_max_len as usize);
                        if result_end <= mem_data.len() {
                            let mem_data = memory.data_mut(&mut caller);
                            mem_data[result_start..result_end]
                                .copy_from_slice(&data[..result_end - result_start]);
                            return data.len() as u32;
                        }
                    }
                    0
                },
            )
            .map_err(|e| {
                FrameworkError::WasmLoad(format!("failed to define wasm_plugin_read_data: {}", e))
            })?;

        // Also define unknown imports as traps for compatibility
        linker
            .define_unknown_imports_as_traps(&module)
            .map_err(|e| FrameworkError::WasmLoad(format!("link failed: {}", e)))?;

        let store_data = StoreData {
            plugin_id,
            read_data_fn: Some(read_data_fn),
            write_data_fn: Some(write_data_fn),
        };

        let mut store = Store::new(engine, store_data);
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

        Ok((instance, store, memory, on_tick, on_event))
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
        // If we have data callbacks, use the enhanced instance creation
        if let (Some(read_fn), Some(write_fn)) = (&self.read_data_fn, &self.write_data_fn) {
            let result = Self::create_instance_with_data_callbacks(
                &self.engine,
                &self.wasm_bytes,
                self.id.clone(),
                read_fn.clone(),
                write_fn.clone(),
            );
            match result {
                Ok((_instance, mut store, memory, _on_tick, on_event)) => {
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
                    log::error!("WASM on_event with callbacks failed: {}", e);
                }
            }
        } else {
            // Fall back to basic instance creation
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

    fn get_state(&self) -> Option<Vec<u8>> {
        let state = self.state.read().ok()?;
        if state.is_empty() {
            None
        } else {
            Some(state.clone())
        }
    }

    fn set_state(&self, state: Vec<u8>) {
        if let Ok(mut guard) = self.state.write() {
            *guard = state;
        }
    }

    fn get_stats(&self) -> Option<crate::wasm::PluginStats> {
        // 创建临时实例来读取统计数据
        let result = Self::create_instance_with_name(&self.engine, &self.wasm_bytes);
        match result {
            Ok((_name, instance, mut store, memory, _on_tick, _on_event)) => {
                // 尝试调用 wasm_plugin_get_stats 和 wasm_plugin_get_stats_len
                if let Ok(get_stats_func) =
                    instance.get_typed_func::<(), u32>(&mut store, "wasm_plugin_get_stats")
                {
                    if let Ok(get_stats_len_func) =
                        instance.get_typed_func::<(), u32>(&mut store, "wasm_plugin_get_stats_len")
                    {
                        if let Ok(stats_ptr) = get_stats_func.call(&mut store, ()) {
                            if let Ok(stats_len) = get_stats_len_func.call(&mut store, ()) {
                                if stats_len >= 12 {
                                    // 从内存读取统计数据
                                    let mem_data = memory.data(&store);
                                    let start = stats_ptr as usize;
                                    let end = start + stats_len as usize;

                                    if end <= mem_data.len() {
                                        let stats_bytes = &mem_data[start..end];
                                        if let Ok(purchase_bytes) = stats_bytes[0..4].try_into() {
                                            if let Ok(heal_bytes) = stats_bytes[4..8].try_into() {
                                                if let Ok(gold_bytes) =
                                                    stats_bytes[8..12].try_into()
                                                {
                                                    return Some(crate::wasm::PluginStats {
                                                        purchase_count: u32::from_le_bytes(
                                                            purchase_bytes,
                                                        ),
                                                        heal_count: u32::from_le_bytes(heal_bytes),
                                                        gold_earned: u32::from_le_bytes(gold_bytes),
                                                    });
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to create instance for stats reading: {}", e);
            }
        }
        None
    }
}
