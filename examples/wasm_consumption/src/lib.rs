//! WASM Consumption Plugin
//!
//! 功能：
//! 1. 定义多种消费项目（普通食物、高级食物、治疗药水）
//! 2. 实现动态定价逻辑（购买次数越多，价格越高）
//! 3. 实现解锁机制（达到购买次数解锁新物品）
//! 4. 提供消费奖励（连续购买获得折扣）

#![no_std]
#![no_main]

// 简单的 panic handler
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

// 插件名称
const PLUGIN_NAME: &[u8] = b"ConsumptionPlugin\0";

#[no_mangle]
pub extern "C" fn wasm_plugin_name() -> *const u8 {
    PLUGIN_NAME.as_ptr()
}

#[no_mangle]
pub extern "C" fn wasm_plugin_name_len() -> usize {
    PLUGIN_NAME.len() - 1
}

// 插件版本
const PLUGIN_VERSION: &[u8] = b"1.0.0\0";

#[no_mangle]
pub extern "C" fn wasm_plugin_version() -> *const u8 {
    PLUGIN_VERSION.as_ptr()
}

#[no_mangle]
pub extern "C" fn wasm_plugin_version_len() -> usize {
    PLUGIN_VERSION.len() - 1
}

// 消费项目定义
#[derive(Clone, Copy)]
struct ShopItem {
    id: u32,
    base_price: u32,
    current_price: u32,
    unlocked: bool,
    purchases_needed: u32, // 解锁所需购买次数
}

impl ShopItem {
    const fn new(id: u32, base_price: u32, purchases_needed: u32) -> Self {
        Self {
            id,
            base_price,
            current_price: base_price,
            unlocked: purchases_needed == 0, // 0 means unlocked by default
            purchases_needed,
        }
    }
}

// 商店状态
#[derive(Clone, Copy)]
struct ShopState {
    total_purchases: u32,
    consecutive_purchases: u32,
    items: [ShopItem; 3],
}

impl ShopState {
    fn new() -> Self {
        Self {
            total_purchases: 0,
            consecutive_purchases: 0,
            items: [
                ShopItem::new(1, 10, 0),  // Basic Food - 10g, 已解锁
                ShopItem::new(2, 25, 5),  // Premium Food - 25g, 需要5次购买
                ShopItem::new(3, 50, 10), // Elixir - 50g, 需要10次购买
            ],
        }
    }

    fn update_prices(&mut self) {
        // 动态定价：每购买一次，价格增加5%
        for item in self.items.iter_mut() {
            if item.unlocked {
                let increase = (item.base_price as f32 * 0.05 * self.total_purchases as f32) as u32;
                item.current_price = item.base_price + increase;
            }
        }

        // 连续购买折扣：连续购买10次后，享受10%折扣
        if self.consecutive_purchases >= 10 {
            for item in self.items.iter_mut() {
                if item.unlocked {
                    item.current_price = (item.current_price as f32 * 0.9) as u32;
                }
            }
        }
    }

    fn check_unlocks(&mut self) {
        for item in self.items.iter_mut() {
            if !item.unlocked && self.total_purchases >= item.purchases_needed {
                item.unlocked = true;
            }
        }
    }

    fn get_unlock_progress(&self) -> [u32; 3] {
        [
            self.items[0].purchases_needed,
            self.items[1].purchases_needed,
            self.items[2].purchases_needed,
        ]
    }
}

// 全局状态
static mut SHOP_STATE: ShopState = ShopState {
    total_purchases: 0,
    consecutive_purchases: 0,
    items: [
        ShopItem {
            id: 1,
            base_price: 10,
            current_price: 10,
            unlocked: true,
            purchases_needed: 0,
        },
        ShopItem {
            id: 2,
            base_price: 25,
            current_price: 25,
            unlocked: false,
            purchases_needed: 5,
        },
        ShopItem {
            id: 3,
            base_price: 50,
            current_price: 50,
            unlocked: false,
            purchases_needed: 10,
        },
    ],
};

// ABI 函数声明
extern "C" {
    fn wasm_plugin_set_data(
        key_ptr: *const u8,
        key_len: usize,
        data_ptr: *const u8,
        data_len: usize,
    );
    fn wasm_plugin_get_config(
        key_ptr: *const u8,
        key_len: usize,
        result_ptr: *mut u8,
        result_max_len: usize,
    ) -> usize;
}

/// 导出商店状态到主机
fn export_shop_state() {
    unsafe {
        // 导出总购买次数
        let key = b"total_purchases";
        let value = SHOP_STATE.total_purchases.to_le_bytes();
        wasm_plugin_set_data(key.as_ptr(), key.len(), value.as_ptr(), value.len());

        // 导出连续购买次数
        let key = b"consecutive_purchases";
        let value = SHOP_STATE.consecutive_purchases.to_le_bytes();
        wasm_plugin_set_data(key.as_ptr(), key.len(), value.as_ptr(), value.len());

        // 导出每个物品的价格和解锁状态
        for i in 0..3 {
            let item = SHOP_STATE.items[i];

            // 价格
            let key = format_price_key(i);
            let value = item.current_price.to_le_bytes();
            unsafe {
                wasm_plugin_set_data(key.as_ptr(), key.len(), value.as_ptr(), value.len());
            }

            // 解锁状态
            let key = format_unlock_key(i);
            let value = [item.unlocked as u8; 1];
            unsafe {
                wasm_plugin_set_data(key.as_ptr(), key.len(), value.as_ptr(), value.len());
            }
        }
    }
}

fn format_price_key(index: usize) -> [u8; 16] {
    let mut key = [0u8; 16];
    let prefix = b"item_";
    let suffix = b"_price";
    key[0..5].copy_from_slice(prefix);
    key[5] = (b'0' + index as u8) as u8;
    key[6..12].copy_from_slice(suffix);
    key
}

fn format_unlock_key(index: usize) -> [u8; 16] {
    let mut key = [0u8; 16];
    let prefix = b"item_";
    let suffix = b"_unlocked";
    key[0..5].copy_from_slice(prefix);
    key[5] = (b'0' + index as u8) as u8;
    key[6..15].copy_from_slice(suffix);
    key
}

/// Called every frame
#[no_mangle]
pub extern "C" fn wasm_plugin_on_tick(entity_id: u64) {
    let _ = entity_id;
    // 每隔一段时间更新价格
    static mut TICK_COUNT: u64 = 0;
    unsafe {
        TICK_COUNT += 1;
        if TICK_COUNT % 60 == 0 {
            // 每60帧更新一次
            export_shop_state();
        }
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

    // 处理购买事件
    unsafe {
        if event_str == "purchase" {
            SHOP_STATE.total_purchases += 1;
            SHOP_STATE.consecutive_purchases += 1;

            // 检查解锁
            SHOP_STATE.check_unlocks();

            // 更新价格
            SHOP_STATE.update_prices();

            // 导出更新后的状态
            export_shop_state();
        } else if event_str == "heal" {
            // 治疗中断连续购买
            SHOP_STATE.consecutive_purchases = 0;
            export_shop_state();
        }
    }

    let _ = entity_id;
}

/// Called when the plugin is loaded
#[no_mangle]
pub extern "C" fn wasm_plugin_on_load() {
    // 初始化商店状态
    unsafe {
        SHOP_STATE = ShopState::new();
    }
}

/// Called when the plugin is unloaded
#[no_mangle]
pub extern "C" fn wasm_plugin_on_unload() {
    // 清理逻辑
}

/// Called when an error occurs
#[no_mangle]
pub extern "C" fn wasm_plugin_on_error(error_code: u32) {
    let _ = error_code;
}
