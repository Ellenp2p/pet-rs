//! Discord Bot API Channel Plugin (WASM)
//!
//! 这是一个示例 WASM 渠道插件，展示如何实现 Discord Bot API 集成。
//!
//! ## 功能
//!
//! - 通过 Discord Bot API 接收和发送消息
//! - 支持 REST API (不支持 Gateway WebSocket)
//! - 支持文本消息
//!
//! ## 构建
//!
//! ```bash
//! cd examples/wasm_channel_discord
//! cargo build --target wasm32-unknown-unknown --release
//! ```
//!
//! ## 导出函数
//!
//! - `wasm_plugin_name` / `wasm_plugin_name_len` - 插件名称
//! - `wasm_plugin_version` / `wasm_plugin_version_len` - 插件版本
//! - `wasm_channel_connect` - 连接到 Discord
//! - `wasm_channel_disconnect` - 断开连接
//! - `wasm_channel_send` - 发送消息
//! - `wasm_channel_poll` - 轮询新消息
//! - `wasm_channel_status` - 获取状态

#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// ========== 插件元数据 ==========

const PLUGIN_NAME: &[u8] = b"discord\0";
const PLUGIN_VERSION: &[u8] = b"1.0.0\0";

#[no_mangle]
pub extern "C" fn wasm_plugin_name() -> *const u8 {
    PLUGIN_NAME.as_ptr()
}

#[no_mangle]
pub extern "C" fn wasm_plugin_name_len() -> usize {
    PLUGIN_NAME.len() - 1
}

#[no_mangle]
pub extern "C" fn wasm_plugin_version() -> *const u8 {
    PLUGIN_VERSION.as_ptr()
}

#[no_mangle]
pub extern "C" fn wasm_plugin_version_len() -> usize {
    PLUGIN_VERSION.len() - 1
}

// ========== 宿主函数声明 ==========

extern "C" {
    fn wasm_plugin_set_data(
        key_ptr: *const u8,
        key_len: usize,
        data_ptr: *const u8,
        data_len: usize,
    );
    fn wasm_plugin_read_data(
        key_ptr: *const u8,
        key_len: usize,
        result_ptr: *mut u8,
        result_max_len: usize,
    ) -> usize;

    fn host_http_request(
        method_ptr: *const u8,
        method_len: usize,
        url_ptr: *const u8,
        url_len: usize,
        headers_ptr: *const u8,
        headers_len: usize,
        body_ptr: *const u8,
        body_len: usize,
        result_ptr: *mut u8,
        result_max_len: usize,
    ) -> usize;

    fn host_get_secret(
        key_ptr: *const u8,
        key_len: usize,
        result_ptr: *mut u8,
        result_max_len: usize,
    ) -> usize;

    fn host_emit_incoming(message_json_ptr: *const u8, message_json_len: usize);

    fn host_http_poll(
        url_ptr: *const u8,
        url_len: usize,
        headers_ptr: *const u8,
        headers_len: usize,
        timeout_ms: u32,
        result_ptr: *mut u8,
        result_max_len: usize,
    ) -> usize;
}

// ========== 全局状态 ==========

static mut BOT_TOKEN: [u8; 256] = [0; 256];
static mut BOT_TOKEN_LEN: usize = 0;
static mut IS_CONNECTED: bool = false;
static mut LAST_MESSAGE_ID: [u8; 64] = [0; 64];
static mut LAST_MESSAGE_ID_LEN: usize = 0;

// ========== 辅助函数 ==========

unsafe fn bytes_to_str(bytes: &[u8]) -> &str {
    core::str::from_utf8_unchecked(bytes)
}

// ========== 生命周期函数 ==========

#[no_mangle]
pub extern "C" fn wasm_plugin_on_tick(_entity_id: u64) {}

#[no_mangle]
pub extern "C" fn wasm_plugin_on_event(
    _entity_id: u64,
    event_ptr: *const u8,
    event_len: usize,
    _data_ptr: *const u8,
    _data_len: usize,
) {
    let event_str = unsafe { bytes_to_str(core::slice::from_raw_parts(event_ptr, event_len)) };

    match event_str {
        "wasm_channel_poll" => {
            poll_messages();
        }
        _ => {}
    }
}

#[no_mangle]
pub extern "C" fn wasm_plugin_on_load() {
    unsafe {
        IS_CONNECTED = false;
        LAST_MESSAGE_ID_LEN = 0;
    }
}

#[no_mangle]
pub extern "C" fn wasm_plugin_on_unload() {
    unsafe {
        IS_CONNECTED = false;
    }
}

#[no_mangle]
pub extern "C" fn wasm_plugin_on_error(_error_code: u32) {}

// ========== 渠道函数 ==========

/// 连接到 Discord
#[no_mangle]
pub extern "C" fn wasm_channel_connect(
    _config_ptr: *const u8,
    _config_len: usize,
    result_ptr: *mut u8,
    result_max_len: usize,
) -> usize {
    unsafe {
        // 获取 bot token
        let key = b"discord_bot_token";
        let token_len = host_get_secret(
            key.as_ptr(),
            key.len(),
            BOT_TOKEN.as_mut_ptr(),
            BOT_TOKEN.len(),
        );

        if token_len > 0 && token_len <= BOT_TOKEN.len() {
            BOT_TOKEN_LEN = token_len;
            IS_CONNECTED = true;

            let response = b"{\"status\":\"connected\",\"channel\":\"discord\"}";
            let copy_len = response.len().min(result_max_len);
            core::ptr::copy_nonoverlapping(response.as_ptr(), result_ptr, copy_len);
            copy_len
        } else {
            let response = b"{\"status\":\"error\",\"message\":\"Failed to get bot token\"}";
            let copy_len = response.len().min(result_max_len);
            core::ptr::copy_nonoverlapping(response.as_ptr(), result_ptr, copy_len);
            copy_len
        }
    }
}

/// 断开连接
#[no_mangle]
pub extern "C" fn wasm_channel_disconnect(result_ptr: *mut u8, result_max_len: usize) -> usize {
    unsafe {
        IS_CONNECTED = false;
        BOT_TOKEN_LEN = 0;
    }

    let response = b"{\"status\":\"disconnected\"}";
    let copy_len = response.len().min(result_max_len);
    unsafe {
        core::ptr::copy_nonoverlapping(response.as_ptr(), result_ptr, copy_len);
    }
    copy_len
}

/// 发送消息
#[no_mangle]
pub extern "C" fn wasm_channel_send(
    _message_ptr: *const u8,
    _message_len: usize,
    result_ptr: *mut u8,
    result_max_len: usize,
) -> usize {
    unsafe {
        if !IS_CONNECTED || BOT_TOKEN_LEN == 0 {
            let response = b"{\"error\":\"Not connected\"}";
            let copy_len = response.len().min(result_max_len);
            core::ptr::copy_nonoverlapping(response.as_ptr(), result_ptr, copy_len);
            return copy_len;
        }
    }

    // Discord REST API: POST /channels/{channel_id}/messages
    // 这里简化处理

    let response = b"{\"message_id\":\"0\",\"status\":\"sent\"}";
    let copy_len = response.len().min(result_max_len);
    unsafe {
        core::ptr::copy_nonoverlapping(response.as_ptr(), result_ptr, copy_len);
    }
    copy_len
}

/// 获取状态
#[no_mangle]
pub extern "C" fn wasm_channel_status(result_ptr: *mut u8, result_max_len: usize) -> usize {
    let connected = unsafe { IS_CONNECTED };

    let response = if connected {
        b"{\"connected\":true,\"pending_messages\":0}"
    } else {
        b"{\"connected\":false,\"pending_messages\":0}"
    };

    let copy_len = response.len().min(result_max_len);
    unsafe {
        core::ptr::copy_nonoverlapping(response.as_ptr(), result_ptr, copy_len);
    }
    copy_len
}

// ========== 内部函数 ==========

/// 轮询 Discord 消息 (简化版本)
fn poll_messages() {
    unsafe {
        if !IS_CONNECTED || BOT_TOKEN_LEN == 0 {
            return;
        }
    }

    // Discord 没有简单的轮询 API，需要使用 Gateway WebSocket
    // 这里简化处理，实际应该实现 Gateway 协议

    // 临时方案：通过 REST API 获取频道消息
    // GET /channels/{channel_id}/messages

    // 简化：发送一个测试响应
    let response = b"{\"messages\":[]}";
    unsafe {
        host_emit_incoming(response.as_ptr(), response.len());
    }
}
