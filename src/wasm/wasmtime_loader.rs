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
    /// Callback to read configuration
    pub read_config_fn: Option<Arc<dyn Fn(&str, &str) -> Option<String> + Send + Sync>>,
    /// Callback to get secrets (API keys)
    pub get_secret_fn: Option<Arc<dyn Fn(&str) -> Option<String> + Send + Sync>>,
    /// Callback to perform HTTP requests (method, url, headers_json, body) -> response_json
    pub http_request_fn:
        Option<Arc<dyn Fn(&str, &str, &str, &[u8]) -> Result<String, String> + Send + Sync>>,
    /// Callback to record usage
    pub record_usage_fn: Option<Arc<dyn Fn(&str, &str, u32, u32, f64) + Send + Sync>>,
    /// Callback to check budget
    pub check_budget_fn: Option<Arc<dyn Fn() -> u32 + Send + Sync>>,
    /// Callback to emit streaming chunk
    pub emit_chunk_fn: Option<Arc<dyn Fn(u32, &str) + Send + Sync>>,
    /// Callback to emit incoming message (for channel plugins)
    pub emit_incoming_fn: Option<Arc<dyn Fn(&str) + Send + Sync>>,
    /// Callback to perform HTTP long-poll (url, headers_json, timeout_ms) -> response_json
    pub http_poll_fn: Option<Arc<dyn Fn(&str, &str, u32) -> Result<String, String> + Send + Sync>>,
}

/// WASM plugin instance wrapper (store-per-call strategy).
#[cfg(feature = "wasm-plugin")]
pub struct WasmtimePlugin {
    id: WasmPluginId,
    name: String,
    version: String,
    wasm_bytes: Vec<u8>,
    engine: Engine,
    state: std::sync::RwLock<Vec<u8>>,
    /// Callback to read plugin data (for inter-plugin communication)
    read_data_fn: Option<Arc<dyn Fn(&str, &str) -> Option<Vec<u8>> + Send + Sync>>,
    /// Callback to write plugin data (for inter-plugin communication)
    write_data_fn: Option<Arc<dyn Fn(&str, &str, Vec<u8>) + Send + Sync>>,
    /// Callback to read configuration
    read_config_fn: Option<Arc<dyn Fn(&str, &str) -> Option<String> + Send + Sync>>,
    /// Callback to get secrets (API keys)
    get_secret_fn: Option<Arc<dyn Fn(&str) -> Option<String> + Send + Sync>>,
    /// Callback to perform HTTP requests
    http_request_fn:
        Option<Arc<dyn Fn(&str, &str, &str, &[u8]) -> Result<String, String> + Send + Sync>>,
    /// Callback to record usage
    record_usage_fn: Option<Arc<dyn Fn(&str, &str, u32, u32, f64) + Send + Sync>>,
    /// Callback to check budget
    check_budget_fn: Option<Arc<dyn Fn() -> u32 + Send + Sync>>,
    /// Callback to emit streaming chunk
    emit_chunk_fn: Option<Arc<dyn Fn(u32, &str) + Send + Sync>>,
    /// Callback to emit incoming message (for channel plugins)
    emit_incoming_fn: Option<Arc<dyn Fn(&str) + Send + Sync>>,
    /// Callback to perform HTTP long-poll
    http_poll_fn: Option<Arc<dyn Fn(&str, &str, u32) -> Result<String, String> + Send + Sync>>,
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

        // Create a temporary instance just to read the name and version
        let (name, version, _instance, store, _memory, _on_tick, _on_event) =
            Self::create_instance_with_name_and_version(&engine, &wasm_bytes)?;
        let id = WasmPluginId::new(override_id.unwrap_or_else(|| name.clone()));

        // Drop the temporary store
        let _ = store;

        Ok(Self {
            id,
            name,
            version,
            wasm_bytes,
            engine,
            state: std::sync::RwLock::new(Vec::new()),
            read_data_fn: None,
            write_data_fn: None,
            read_config_fn: None,
            get_secret_fn: None,
            http_request_fn: None,
            record_usage_fn: None,
            check_budget_fn: None,
            emit_chunk_fn: None,
            emit_incoming_fn: None,
            http_poll_fn: None,
        })
    }

    /// Load a WASM plugin with data callbacks for inter-plugin communication.
    pub fn load_with_callbacks(
        wasm_path: &std::path::Path,
        override_id: Option<String>,
        read_data_fn: Arc<dyn Fn(&str, &str) -> Option<Vec<u8>> + Send + Sync>,
        write_data_fn: Arc<dyn Fn(&str, &str, Vec<u8>) + Send + Sync>,
        read_config_fn: Arc<dyn Fn(&str, &str) -> Option<String> + Send + Sync>,
    ) -> Result<Self, FrameworkError> {
        let wasm_bytes = fs::read(wasm_path)
            .map_err(|e| FrameworkError::WasmLoad(format!("failed to read file: {}", e)))?;

        let engine = Engine::default();

        // Create a temporary instance just to read the name and version
        let (name, version, _instance, store, _memory, _on_tick, _on_event) =
            Self::create_instance_with_name_and_version(&engine, &wasm_bytes)?;
        let id = WasmPluginId::new(override_id.unwrap_or_else(|| name.clone()));

        // Drop the temporary store
        let _ = store;

        Ok(Self {
            id,
            name,
            version,
            wasm_bytes,
            engine,
            state: std::sync::RwLock::new(Vec::new()),
            read_data_fn: Some(read_data_fn),
            write_data_fn: Some(write_data_fn),
            read_config_fn: Some(read_config_fn),
            get_secret_fn: None,
            http_request_fn: None,
            record_usage_fn: None,
            check_budget_fn: None,
            emit_chunk_fn: None,
            emit_incoming_fn: None,
            http_poll_fn: None,
        })
    }

    /// Load a WASM plugin with AI provider callbacks.
    #[allow(clippy::too_many_arguments)]
    pub fn load_with_ai_callbacks(
        wasm_path: &std::path::Path,
        override_id: Option<String>,
        read_data_fn: Arc<dyn Fn(&str, &str) -> Option<Vec<u8>> + Send + Sync>,
        write_data_fn: Arc<dyn Fn(&str, &str, Vec<u8>) + Send + Sync>,
        read_config_fn: Arc<dyn Fn(&str, &str) -> Option<String> + Send + Sync>,
        get_secret_fn: Arc<dyn Fn(&str) -> Option<String> + Send + Sync>,
        http_request_fn: Arc<
            dyn Fn(&str, &str, &str, &[u8]) -> Result<String, String> + Send + Sync,
        >,
        record_usage_fn: Arc<dyn Fn(&str, &str, u32, u32, f64) + Send + Sync>,
        check_budget_fn: Arc<dyn Fn() -> u32 + Send + Sync>,
        emit_chunk_fn: Arc<dyn Fn(u32, &str) + Send + Sync>,
    ) -> Result<Self, FrameworkError> {
        let wasm_bytes = fs::read(wasm_path)
            .map_err(|e| FrameworkError::WasmLoad(format!("failed to read file: {}", e)))?;

        let engine = Engine::default();

        // Create a temporary instance just to read the name and version
        let (name, version, _instance, store, _memory, _on_tick, _on_event) =
            Self::create_instance_with_name_and_version(&engine, &wasm_bytes)?;
        let id = WasmPluginId::new(override_id.unwrap_or_else(|| name.clone()));

        // Drop the temporary store
        let _ = store;

        Ok(Self {
            id,
            name,
            version,
            wasm_bytes,
            engine,
            state: std::sync::RwLock::new(Vec::new()),
            read_data_fn: Some(read_data_fn),
            write_data_fn: Some(write_data_fn),
            read_config_fn: Some(read_config_fn),
            get_secret_fn: Some(get_secret_fn),
            http_request_fn: Some(http_request_fn),
            record_usage_fn: Some(record_usage_fn),
            check_budget_fn: Some(check_budget_fn),
            emit_chunk_fn: Some(emit_chunk_fn),
            emit_incoming_fn: None,
            http_poll_fn: None,
        })
    }

    /// Load a WASM plugin with channel plugin callbacks.
    #[allow(clippy::too_many_arguments)]
    pub fn load_with_channel_callbacks(
        wasm_path: &std::path::Path,
        override_id: Option<String>,
        read_data_fn: Arc<dyn Fn(&str, &str) -> Option<Vec<u8>> + Send + Sync>,
        write_data_fn: Arc<dyn Fn(&str, &str, Vec<u8>) + Send + Sync>,
        read_config_fn: Arc<dyn Fn(&str, &str) -> Option<String> + Send + Sync>,
        get_secret_fn: Arc<dyn Fn(&str) -> Option<String> + Send + Sync>,
        http_request_fn: Arc<
            dyn Fn(&str, &str, &str, &[u8]) -> Result<String, String> + Send + Sync,
        >,
        emit_incoming_fn: Arc<dyn Fn(&str) + Send + Sync>,
        http_poll_fn: Arc<dyn Fn(&str, &str, u32) -> Result<String, String> + Send + Sync>,
    ) -> Result<Self, FrameworkError> {
        let wasm_bytes = fs::read(wasm_path)
            .map_err(|e| FrameworkError::WasmLoad(format!("failed to read file: {}", e)))?;

        let engine = Engine::default();

        let (name, version, _instance, store, _memory, _on_tick, _on_event) =
            Self::create_instance_with_name_and_version(&engine, &wasm_bytes)?;
        let id = WasmPluginId::new(override_id.unwrap_or_else(|| name.clone()));

        let _ = store;

        Ok(Self {
            id,
            name,
            version,
            wasm_bytes,
            engine,
            state: std::sync::RwLock::new(Vec::new()),
            read_data_fn: Some(read_data_fn),
            write_data_fn: Some(write_data_fn),
            read_config_fn: Some(read_config_fn),
            get_secret_fn: Some(get_secret_fn),
            http_request_fn: Some(http_request_fn),
            record_usage_fn: None,
            check_budget_fn: None,
            emit_chunk_fn: None,
            emit_incoming_fn: Some(emit_incoming_fn),
            http_poll_fn: Some(http_poll_fn),
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

    /// Create instance and read name and version in one go.
    fn create_instance_with_name_and_version(
        engine: &Engine,
        wasm_bytes: &[u8],
    ) -> Result<
        (
            String,
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

        // Read plugin version (optional)
        let version = if let Ok(version_ptr_func) =
            instance.get_typed_func::<(), u32>(&mut store, "wasm_plugin_version")
        {
            if let Ok(version_len_func) =
                instance.get_typed_func::<(), u32>(&mut store, "wasm_plugin_version_len")
            {
                if let Ok(version_ptr) = version_ptr_func.call(&mut store, ()) {
                    if let Ok(version_len) = version_len_func.call(&mut store, ()) {
                        let data = memory.data(&store);
                        let start = version_ptr as usize;
                        let end = start + version_len as usize;
                        if end <= data.len() {
                            let version_bytes = &data[start..end];
                            String::from_utf8(version_bytes.to_vec())
                                .unwrap_or_else(|_| "0.0.0".to_string())
                        } else {
                            "0.0.0".to_string()
                        }
                    } else {
                        "0.0.0".to_string()
                    }
                } else {
                    "0.0.0".to_string()
                }
            } else {
                "0.0.0".to_string()
            }
        } else {
            "0.0.0".to_string()
        };

        Ok((name, version, instance, store, memory, on_tick, on_event))
    }

    /// Create instance with custom host functions for data read/write.
    #[allow(dead_code)]
    fn create_instance_with_data_callbacks(
        engine: &Engine,
        wasm_bytes: &[u8],
        plugin_id: WasmPluginId,
        read_data_fn: Arc<dyn Fn(&str, &str) -> Option<Vec<u8>> + Send + Sync>,
        write_data_fn: Arc<dyn Fn(&str, &str, Vec<u8>) + Send + Sync>,
        read_config_fn: Arc<dyn Fn(&str, &str) -> Option<String> + Send + Sync>,
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
        Self::create_instance_with_all_callbacks(
            engine,
            wasm_bytes,
            plugin_id,
            read_data_fn,
            write_data_fn,
            read_config_fn,
            None, // get_secret_fn
            None, // http_request_fn
            None, // record_usage_fn
            None, // check_budget_fn
            None, // emit_chunk_fn
        )
    }

    /// Create instance with all callbacks including AI provider support.
    #[allow(clippy::too_many_arguments)]
    fn create_instance_with_all_callbacks(
        engine: &Engine,
        wasm_bytes: &[u8],
        plugin_id: WasmPluginId,
        read_data_fn: Arc<dyn Fn(&str, &str) -> Option<Vec<u8>> + Send + Sync>,
        write_data_fn: Arc<dyn Fn(&str, &str, Vec<u8>) + Send + Sync>,
        read_config_fn: Arc<dyn Fn(&str, &str) -> Option<String> + Send + Sync>,
        get_secret_fn: Option<Arc<dyn Fn(&str) -> Option<String> + Send + Sync>>,
        http_request_fn: Option<
            Arc<dyn Fn(&str, &str, &str, &[u8]) -> Result<String, String> + Send + Sync>,
        >,
        record_usage_fn: Option<Arc<dyn Fn(&str, &str, u32, u32, f64) + Send + Sync>>,
        check_budget_fn: Option<Arc<dyn Fn() -> u32 + Send + Sync>>,
        emit_chunk_fn: Option<Arc<dyn Fn(u32, &str) + Send + Sync>>,
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

                    let key_start = key_ptr as usize;
                    let key_end = key_start + key_len as usize;
                    if key_end > mem_data.len() {
                        return;
                    }
                    let key_bytes = &mem_data[key_start..key_end];
                    let key = core::str::from_utf8(key_bytes).unwrap_or("");

                    let data_start = data_ptr as usize;
                    let data_end = data_start + data_len as usize;
                    if data_end > mem_data.len() {
                        return;
                    }
                    let data = mem_data[data_start..data_end].to_vec();

                    write_data_fn_clone(key, key, data);
                },
            )
            .map_err(|e| {
                FrameworkError::WasmLoad(format!("failed to define wasm_plugin_set_data: {}", e))
            })?;

        // Define host function: wasm_plugin_read_data
        let read_data_fn_clone = read_data_fn.clone();
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

                    let key_start = key_ptr as usize;
                    let key_end = key_start + key_len as usize;
                    if key_end > mem_data.len() {
                        return 0;
                    }
                    let key_bytes = &mem_data[key_start..key_end];
                    let key = core::str::from_utf8(key_bytes).unwrap_or("");

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

        // Define host function: wasm_plugin_get_config
        let read_config_fn_clone = read_config_fn.clone();
        linker
            .func_wrap(
                "env",
                "wasm_plugin_get_config",
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

                    let key_start = key_ptr as usize;
                    let key_end = key_start + key_len as usize;
                    if key_end > mem_data.len() {
                        return 0;
                    }
                    let key_bytes = &mem_data[key_start..key_end];
                    let key = core::str::from_utf8(key_bytes).unwrap_or("");

                    if let Some(config_value) = read_config_fn_clone(key, key) {
                        let config_bytes = config_value.as_bytes();
                        let result_start = result_ptr as usize;
                        let result_end =
                            result_start + config_bytes.len().min(result_max_len as usize);
                        if result_end <= mem_data.len() {
                            let mem_data = memory.data_mut(&mut caller);
                            mem_data[result_start..result_end]
                                .copy_from_slice(&config_bytes[..result_end - result_start]);
                            return config_bytes.len() as u32;
                        }
                    }
                    0
                },
            )
            .map_err(|e| {
                FrameworkError::WasmLoad(format!("failed to define wasm_plugin_get_config: {}", e))
            })?;

        // Define host function: host_get_secret(key_ptr, key_len, result_ptr, result_max_len) -> u32
        if let Some(get_secret_fn) = get_secret_fn.clone() {
            linker
                .func_wrap(
                    "env",
                    "host_get_secret",
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

                        let key_start = key_ptr as usize;
                        let key_end = key_start + key_len as usize;
                        if key_end > mem_data.len() {
                            return 0;
                        }
                        let key_bytes = &mem_data[key_start..key_end];
                        let key = core::str::from_utf8(key_bytes).unwrap_or("");

                        if let Some(secret) = get_secret_fn(key) {
                            let secret_bytes = secret.as_bytes();
                            let result_start = result_ptr as usize;
                            let result_end =
                                result_start + secret_bytes.len().min(result_max_len as usize);
                            if result_end <= mem_data.len() {
                                let mem_data = memory.data_mut(&mut caller);
                                mem_data[result_start..result_end]
                                    .copy_from_slice(&secret_bytes[..result_end - result_start]);
                                return secret_bytes.len() as u32;
                            }
                        }
                        0
                    },
                )
                .map_err(|e| {
                    FrameworkError::WasmLoad(format!("failed to define host_get_secret: {}", e))
                })?;
        }

        // Define host function: host_http_request(method_ptr, method_len, url_ptr, url_len, headers_ptr, headers_len, body_ptr, body_len, result_ptr, result_max_len) -> u32
        if let Some(http_request_fn) = http_request_fn.clone() {
            linker
                .func_wrap(
                    "env",
                    "host_http_request",
                    move |mut caller: Caller<'_, StoreData>,
                          method_ptr: u32,
                          method_len: u32,
                          url_ptr: u32,
                          url_len: u32,
                          headers_ptr: u32,
                          headers_len: u32,
                          body_ptr: u32,
                          body_len: u32,
                          result_ptr: u32,
                          result_max_len: u32|
                          -> u32 {
                        let memory = caller
                            .get_export("memory")
                            .and_then(|e| e.into_memory())
                            .unwrap();
                        let mem_data = memory.data(&caller);

                        // Read method
                        let method_start = method_ptr as usize;
                        let method_end = method_start + method_len as usize;
                        if method_end > mem_data.len() {
                            return 0;
                        }
                        let method = core::str::from_utf8(&mem_data[method_start..method_end])
                            .unwrap_or("GET");

                        // Read URL
                        let url_start = url_ptr as usize;
                        let url_end = url_start + url_len as usize;
                        if url_end > mem_data.len() {
                            return 0;
                        }
                        let url = core::str::from_utf8(&mem_data[url_start..url_end]).unwrap_or("");

                        // Read headers (JSON)
                        let headers_start = headers_ptr as usize;
                        let headers_end = headers_start + headers_len as usize;
                        if headers_end > mem_data.len() {
                            return 0;
                        }
                        let headers = core::str::from_utf8(&mem_data[headers_start..headers_end])
                            .unwrap_or("{}");

                        // Read body
                        let body_start = body_ptr as usize;
                        let body_end = body_start + body_len as usize;
                        if body_end > mem_data.len() {
                            return 0;
                        }
                        let body = &mem_data[body_start..body_end];

                        // Call the HTTP request function
                        match http_request_fn(method, url, headers, body) {
                            Ok(response) => {
                                let response_bytes = response.as_bytes();
                                let result_start = result_ptr as usize;
                                let result_end = result_start
                                    + response_bytes.len().min(result_max_len as usize);
                                if result_end <= mem_data.len() {
                                    let mem_data = memory.data_mut(&mut caller);
                                    mem_data[result_start..result_end].copy_from_slice(
                                        &response_bytes[..result_end - result_start],
                                    );
                                    return response_bytes.len() as u32;
                                }
                                0
                            }
                            Err(_) => 0,
                        }
                    },
                )
                .map_err(|e| {
                    FrameworkError::WasmLoad(format!("failed to define host_http_request: {}", e))
                })?;
        }

        // Define host function: host_record_usage(provider_ptr, provider_len, model_ptr, model_len, input_tokens, output_tokens, cost_bytes_ptr)
        if let Some(record_usage_fn) = record_usage_fn.clone() {
            linker
                .func_wrap(
                    "env",
                    "host_record_usage",
                    move |mut caller: Caller<'_, StoreData>,
                          provider_ptr: u32,
                          provider_len: u32,
                          model_ptr: u32,
                          model_len: u32,
                          input_tokens: u32,
                          output_tokens: u32,
                          cost_bytes_ptr: u32| {
                        let memory = caller
                            .get_export("memory")
                            .and_then(|e| e.into_memory())
                            .unwrap();
                        let mem_data = memory.data(&caller);

                        // Read provider
                        let provider_start = provider_ptr as usize;
                        let provider_end = provider_start + provider_len as usize;
                        if provider_end > mem_data.len() {
                            return;
                        }
                        let provider =
                            core::str::from_utf8(&mem_data[provider_start..provider_end])
                                .unwrap_or("");

                        // Read model
                        let model_start = model_ptr as usize;
                        let model_end = model_start + model_len as usize;
                        if model_end > mem_data.len() {
                            return;
                        }
                        let model =
                            core::str::from_utf8(&mem_data[model_start..model_end]).unwrap_or("");

                        // Read cost (f64 as 8 bytes)
                        let cost_start = cost_bytes_ptr as usize;
                        let cost_end = cost_start + 8;
                        if cost_end > mem_data.len() {
                            return;
                        }
                        let cost_bytes: [u8; 8] =
                            mem_data[cost_start..cost_end].try_into().unwrap_or([0; 8]);
                        let cost = f64::from_le_bytes(cost_bytes);

                        record_usage_fn(provider, model, input_tokens, output_tokens, cost);
                    },
                )
                .map_err(|e| {
                    FrameworkError::WasmLoad(format!("failed to define host_record_usage: {}", e))
                })?;
        }

        // Define host function: host_check_budget() -> u32
        if let Some(check_budget_fn) = check_budget_fn.clone() {
            linker
                .func_wrap(
                    "env",
                    "host_check_budget",
                    move |_: Caller<'_, StoreData>| -> u32 { check_budget_fn() },
                )
                .map_err(|e| {
                    FrameworkError::WasmLoad(format!("failed to define host_check_budget: {}", e))
                })?;
        }

        // Define host function: host_emit_chunk(callback_id, chunk_ptr, chunk_len)
        if let Some(emit_chunk_fn) = emit_chunk_fn.clone() {
            linker
                .func_wrap(
                    "env",
                    "host_emit_chunk",
                    move |mut caller: Caller<'_, StoreData>,
                          callback_id: u32,
                          chunk_ptr: u32,
                          chunk_len: u32| {
                        let memory = caller
                            .get_export("memory")
                            .and_then(|e| e.into_memory())
                            .unwrap();
                        let mem_data = memory.data(&caller);

                        let chunk_start = chunk_ptr as usize;
                        let chunk_end = chunk_start + chunk_len as usize;
                        if chunk_end > mem_data.len() {
                            return;
                        }
                        let chunk =
                            core::str::from_utf8(&mem_data[chunk_start..chunk_end]).unwrap_or("");

                        emit_chunk_fn(callback_id, chunk);
                    },
                )
                .map_err(|e| {
                    FrameworkError::WasmLoad(format!("failed to define host_emit_chunk: {}", e))
                })?;
        }

        // Also define unknown imports as traps for compatibility
        linker
            .define_unknown_imports_as_traps(&module)
            .map_err(|e| FrameworkError::WasmLoad(format!("link failed: {}", e)))?;

        let store_data = StoreData {
            plugin_id,
            read_data_fn: Some(read_data_fn),
            write_data_fn: Some(write_data_fn),
            read_config_fn: Some(read_config_fn),
            get_secret_fn,
            http_request_fn,
            record_usage_fn,
            check_budget_fn,
            emit_chunk_fn,
            emit_incoming_fn: None,
            http_poll_fn: None,
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

    fn version(&self) -> &str {
        &self.version
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
        if let (Some(read_fn), Some(write_fn), Some(config_fn)) = (
            &self.read_data_fn,
            &self.write_data_fn,
            &self.read_config_fn,
        ) {
            let result = Self::create_instance_with_all_callbacks(
                &self.engine,
                &self.wasm_bytes,
                self.id.clone(),
                read_fn.clone(),
                write_fn.clone(),
                config_fn.clone(),
                self.get_secret_fn.clone(),
                self.http_request_fn.clone(),
                self.record_usage_fn.clone(),
                self.check_budget_fn.clone(),
                self.emit_chunk_fn.clone(),
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

    fn on_load(&self) -> Result<(), crate::error::FrameworkError> {
        // 尝试调用 wasm_plugin_on_load 函数（如果存在）
        let result = Self::create_instance_with_name(&self.engine, &self.wasm_bytes);
        match result {
            Ok((_name, instance, mut store, _memory, _on_tick, _on_event)) => {
                if let Ok(on_load_func) =
                    instance.get_typed_func::<(), ()>(&mut store, "wasm_plugin_on_load")
                {
                    if let Err(e) = on_load_func.call(&mut store, ()) {
                        log::warn!("wasm_plugin_on_load call failed: {}", e);
                    }
                }
                Ok(())
            }
            Err(e) => Err(crate::error::FrameworkError::WasmLoad(format!(
                "Failed to create instance for on_load: {}",
                e
            ))),
        }
    }

    fn on_unload(&self) -> Result<(), crate::error::FrameworkError> {
        // 尝试调用 wasm_plugin_on_unload 函数（如果存在）
        let result = Self::create_instance_with_name(&self.engine, &self.wasm_bytes);
        match result {
            Ok((_name, instance, mut store, _memory, _on_tick, _on_event)) => {
                if let Ok(on_unload_func) =
                    instance.get_typed_func::<(), ()>(&mut store, "wasm_plugin_on_unload")
                {
                    if let Err(e) = on_unload_func.call(&mut store, ()) {
                        log::warn!("wasm_plugin_on_unload call failed: {}", e);
                    }
                }
                Ok(())
            }
            Err(e) => Err(crate::error::FrameworkError::WasmLoad(format!(
                "Failed to create instance for on_unload: {}",
                e
            ))),
        }
    }

    fn on_error(&self, error: &crate::error::FrameworkError) {
        // 尝试调用 wasm_plugin_on_error 函数（如果存在）
        let result = Self::create_instance_with_name(&self.engine, &self.wasm_bytes);
        match result {
            Ok((_name, instance, mut store, memory, _on_tick, _on_event)) => {
                if let Ok(on_error_func) =
                    instance.get_typed_func::<(u32,), ()>(&mut store, "wasm_plugin_on_error")
                {
                    // 将错误信息写入内存
                    let error_msg = error.to_string();
                    let error_bytes = error_msg.as_bytes();
                    let mem_data = memory.data_mut(&mut store);
                    let scratch_start = 0;
                    let scratch_end = scratch_start + Self::SCRATCH_SIZE;

                    if error_bytes.len() <= scratch_end - scratch_start {
                        mem_data[scratch_start..scratch_start + error_bytes.len()]
                            .copy_from_slice(error_bytes);

                        // 调用 on_error，传递错误代码 1（通用错误）
                        if let Err(e) = on_error_func.call(&mut store, (1,)) {
                            log::warn!("wasm_plugin_on_error call failed: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to create instance for on_error: {}", e);
            }
        }
    }
}
