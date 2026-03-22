//! Example WASM plugin for pet-rs framework.
//!
//! This is a no_std WASM plugin that exports the required ABI functions.
//!
//! # Building
//!
//! ```bash
//! cd examples/wasm_hooks
//! cargo build --target wasm32-unknown-unknown --release
//! cd ../..
//! # Result: examples/wasm_hooks/target/wasm32-unknown-unknown/release/wasm_hooks.wasm
//! ```
//!
//! # ABI
//!
//! This plugin exports:
//! - `wasm_plugin_name()` - returns pointer to name string
//! - `wasm_plugin_name_len()` - returns length of name string
//! - `wasm_plugin_on_tick(entity_id)` - called every frame
//! - `wasm_plugin_on_event(entity_id, event_ptr, event_len, data_ptr, data_len)` - called on events

#![no_std]
#![no_main]

// Simple panic handler that aborts
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

// Plugin name stored in a static buffer
const PLUGIN_NAME: &[u8] = b"ExampleWasmPlugin\0";

#[no_mangle]
pub extern "C" fn wasm_plugin_name() -> *const u8 {
    PLUGIN_NAME.as_ptr()
}

#[no_mangle]
pub extern "C" fn wasm_plugin_name_len() -> usize {
    PLUGIN_NAME.len() - 1 // Exclude null terminator
}

/// Called every frame by the framework
#[no_mangle]
pub extern "C" fn wasm_plugin_on_tick(entity_id: u64) {
    // Simple example: could log to stdout via wasm logging
    // In this no_std version, we do nothing visible
    // A more advanced version would write to memory for host to read
    let _ = entity_id;
}

/// Called when an event occurs
#[no_mangle]
pub extern "C" fn wasm_plugin_on_event(
    entity_id: u64,
    event_ptr: *const u8,
    event_len: usize,
    data_ptr: *const u8,
    data_len: usize,
) {
    // Example: read event and data strings from memory
    // In a real plugin, you'd process these strings
    let _ = entity_id;
    let _ = event_ptr;
    let _ = event_len;
    let _ = data_ptr;
    let _ = data_len;
}
