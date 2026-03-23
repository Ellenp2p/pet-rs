//! WASM Reader Plugin
//!
//! This plugin demonstrates inter-plugin communication by reading data from other plugins.

#![no_std]
#![no_main]

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

const PLUGIN_NAME: &[u8] = b"ReaderPlugin\0";

#[no_mangle]
pub extern "C" fn wasm_plugin_name() -> *const u8 {
    PLUGIN_NAME.as_ptr()
}

#[no_mangle]
pub extern "C" fn wasm_plugin_name_len() -> usize {
    PLUGIN_NAME.len() - 1
}

// ABI 函数声明（主机实现）
extern "C" {
    fn wasm_plugin_read_data(
        key_ptr: *const u8,
        key_len: usize,
        result_ptr: *mut u8,
        result_max_len: usize,
    ) -> usize;
}

// Plugin state to track requests
struct ReaderState {
    last_purchase_count: u32,
    last_heal_count: u32,
    last_gold_earned: u32,
    read_count: u32,
}

static mut STATE: ReaderState = ReaderState {
    last_purchase_count: 0,
    last_heal_count: 0,
    last_gold_earned: 0,
    read_count: 0,
};

/// 读取其他插件的数据
fn read_other_plugin_data(key: &str) -> Option<u32> {
    unsafe {
        let mut buffer = [0u8; 4];
        let result =
            wasm_plugin_read_data(key.as_ptr(), key.len(), buffer.as_mut_ptr(), buffer.len());
        if result == 4 {
            Some(u32::from_le_bytes(buffer))
        } else {
            None
        }
    }
}

#[no_mangle]
pub extern "C" fn wasm_plugin_on_tick(entity_id: u64) {
    // 每隔几帧读取其他插件的数据
    unsafe {
        if entity_id % 10 == 0 {
            // 读取 purchase_count
            if let Some(count) = read_other_plugin_data("purchase_count") {
                STATE.last_purchase_count = count;
            }

            // 读取 heal_count
            if let Some(count) = read_other_plugin_data("heal_count") {
                STATE.last_heal_count = count;
            }

            // 读取 gold_earned
            if let Some(amount) = read_other_plugin_data("gold_earned") {
                STATE.last_gold_earned = amount;
            }

            STATE.read_count += 1;
        }
    }
}

#[no_mangle]
pub extern "C" fn wasm_plugin_on_event(
    entity_id: u64,
    event_ptr: *const u8,
    event_len: usize,
    data_ptr: *const u8,
    data_len: usize,
) {
    // Read event string
    let event_str = if event_ptr != core::ptr::null() && event_len > 0 {
        unsafe {
            let bytes = core::slice::from_raw_parts(event_ptr, event_len);
            core::str::from_utf8(bytes).unwrap_or("")
        }
    } else {
        ""
    };

    // 处理事件
    unsafe {
        match event_str {
            "purchase" | "heal" | "gain_gold" => {
                // 当有事件发生时，立即读取最新的统计数据
                if let Some(count) = read_other_plugin_data("purchase_count") {
                    STATE.last_purchase_count = count;
                }
                if let Some(count) = read_other_plugin_data("heal_count") {
                    STATE.last_heal_count = count;
                }
                if let Some(amount) = read_other_plugin_data("gold_earned") {
                    STATE.last_gold_earned = amount;
                }
            }
            _ => {}
        }
    }

    let _ = entity_id;
    let _ = data_ptr;
    let _ = data_len;
}
