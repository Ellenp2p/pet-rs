//! WASM 统计插件
//!
//! 功能：
//! 1. 统计购买次数、治疗次数、金币获得
//! 2. 将统计数据存储在插件内部状态中
//! 3. 主机通过 get_state 读取统计数据

#![no_std]
#![no_main]

// 简单的 panic handler
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

// 插件名称
const PLUGIN_NAME: &[u8] = b"StatsPlugin\0";

#[no_mangle]
pub extern "C" fn wasm_plugin_name() -> *const u8 {
    PLUGIN_NAME.as_ptr()
}

#[no_mangle]
pub extern "C" fn wasm_plugin_name_len() -> usize {
    PLUGIN_NAME.len() - 1
}

// 统计数据结构
#[derive(Clone, Copy)]
struct Stats {
    purchase_count: u32,
    heal_count: u32,
    gold_earned: u32,
}

impl Stats {
    fn new() -> Self {
        Self {
            purchase_count: 0,
            heal_count: 0,
            gold_earned: 0,
        }
    }

    fn to_bytes(&self) -> [u8; 12] {
        let mut bytes = [0u8; 12];
        bytes[0..4].copy_from_slice(&self.purchase_count.to_le_bytes());
        bytes[4..8].copy_from_slice(&self.heal_count.to_le_bytes());
        bytes[8..12].copy_from_slice(&self.gold_earned.to_le_bytes());
        bytes
    }

    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 12 {
            return None;
        }
        Some(Self {
            purchase_count: u32::from_le_bytes(bytes[0..4].try_into().ok()?),
            heal_count: u32::from_le_bytes(bytes[4..8].try_into().ok()?),
            gold_earned: u32::from_le_bytes(bytes[8..12].try_into().ok()?),
        })
    }
}

// 全局状态（在 no_std 环境中使用静态变量）
static mut STATS: Stats = Stats {
    purchase_count: 0,
    heal_count: 0,
    gold_earned: 0,
};

/// Called every frame by the framework
#[no_mangle]
pub extern "C" fn wasm_plugin_on_tick(entity_id: u64) {
    let _ = entity_id;
}

// ABI 函数声明（主机实现）
extern "C" {
    fn wasm_plugin_set_data(
        key_ptr: *const u8,
        key_len: usize,
        data_ptr: *const u8,
        data_len: usize,
    );
}

/// 导出统计数据到主机
fn export_stats() {
    unsafe {
        let stats = STATS.to_bytes();

        // 导出 purchase_count
        let key = b"purchase_count";
        let value = STATS.purchase_count.to_le_bytes();
        wasm_plugin_set_data(key.as_ptr(), key.len(), value.as_ptr(), value.len());

        // 导出 heal_count
        let key = b"heal_count";
        let value = STATS.heal_count.to_le_bytes();
        wasm_plugin_set_data(key.as_ptr(), key.len(), value.as_ptr(), value.len());

        // 导出 gold_earned
        let key = b"gold_earned";
        let value = STATS.gold_earned.to_le_bytes();
        wasm_plugin_set_data(key.as_ptr(), key.len(), value.as_ptr(), value.len());
    }
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
    // 读取事件字符串
    let event_str = if event_ptr != core::ptr::null() && event_len > 0 {
        unsafe {
            let bytes = core::slice::from_raw_parts(event_ptr, event_len);
            core::str::from_utf8(bytes).unwrap_or("")
        }
    } else {
        ""
    };

    // 读取数据字符串
    let data_str = if data_ptr != core::ptr::null() && data_len > 0 {
        unsafe {
            let bytes = core::slice::from_raw_parts(data_ptr, data_len);
            core::str::from_utf8(bytes).unwrap_or("")
        }
    } else {
        ""
    };

    // 处理事件并更新统计
    unsafe {
        match event_str {
            "purchase" => {
                STATS.purchase_count += 1;
            }
            "heal" => {
                STATS.heal_count += 1;
            }
            "gain_gold" => {
                // 解析金币数量
                if let Ok(amount) = data_str.parse::<u32>() {
                    STATS.gold_earned += amount;
                }
            }
            _ => {}
        }
    }

    // 导出统计数据到主机
    export_stats();

    let _ = entity_id;
}

/// 提供统计数据读取接口（通过内存）
/// 主机可以通过读取内存特定位置来获取统计数据
#[no_mangle]
pub extern "C" fn wasm_plugin_get_stats() -> *const u8 {
    unsafe { STATS.to_bytes().as_ptr() }
}

#[no_mangle]
pub extern "C" fn wasm_plugin_get_stats_len() -> usize {
    12 // Stats 结构体的大小
}
