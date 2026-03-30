//! Telegram Bot API Channel Plugin (WASM)
//!
//! 这是一个示例 WASM 渠道插件，展示如何实现 Telegram Bot API 集成。
//!
//! ## 功能
//!
//! - 通过 Telegram Bot API 接收和发送消息
//! - 支持长轮询 (getUpdates)
//! - 支持文本消息、命令、Markdown 格式
//!
//! ## 构建
//!
//! ```bash
//! cd examples/wasm_channel_telegram
//! cargo build --target wasm32-unknown-unknown --release
//! ```
//!
//! ## 导出函数
//!
//! - `wasm_plugin_name` / `wasm_plugin_name_len` - 插件名称
//! - `wasm_plugin_version` / `wasm_plugin_version_len` - 插件版本
//! - `wasm_plugin_on_tick` - 定时回调
//! - `wasm_plugin_on_event` - 事件处理
//! - `wasm_channel_connect` - 连接到 Telegram
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

const PLUGIN_NAME: &[u8] = b"telegram\0";
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
    fn wasm_plugin_get_config(
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
static mut LAST_UPDATE_ID: i64 = 0;
static mut IS_CONNECTED: bool = false;

// ========== 辅助函数 ==========

/// 将字节数组转换为字符串（不安全，需要确保是有效 UTF-8）
unsafe fn bytes_to_str(bytes: &[u8]) -> &str {
    core::str::from_utf8_unchecked(bytes)
}

/// 构建 Telegram API URL
fn build_api_url(token: &[u8], method: &[u8], buf: &mut [u8]) -> usize {
    let prefix = b"https://api.telegram.org/bot";
    let mut pos = 0;

    // 写入前缀
    for &b in prefix {
        if pos < buf.len() {
            buf[pos] = b;
            pos += 1;
        }
    }

    // 写入 token
    for &b in token {
        if pos < buf.len() {
            buf[pos] = b;
            pos += 1;
        }
    }

    // 写入斜杠
    if pos < buf.len() {
        buf[pos] = b'/';
        pos += 1;
    }

    // 写入方法名
    for &b in method {
        if pos < buf.len() {
            buf[pos] = b;
            pos += 1;
        }
    }

    pos
}

// ========== 生命周期函数 ==========

#[no_mangle]
pub extern "C" fn wasm_plugin_on_tick(_entity_id: u64) {
    // 定期执行任务
}

#[no_mangle]
pub extern "C" fn wasm_plugin_on_event(
    _entity_id: u64,
    event_ptr: *const u8,
    event_len: usize,
    data_ptr: *const u8,
    data_len: usize,
) {
    let event_str = unsafe { bytes_to_str(core::slice::from_raw_parts(event_ptr, event_len)) };

    match event_str {
        "wasm_channel_poll" => {
            // 轮询新消息
            poll_updates();
        }
        _ => {}
    }
}

#[no_mangle]
pub extern "C" fn wasm_plugin_on_load() {
    unsafe {
        IS_CONNECTED = false;
        LAST_UPDATE_ID = 0;
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

/// 连接到 Telegram
#[no_mangle]
pub extern "C" fn wasm_channel_connect(
    config_ptr: *const u8,
    config_len: usize,
    result_ptr: *mut u8,
    result_max_len: usize,
) -> usize {
    let config_bytes = unsafe { core::slice::from_raw_parts(config_ptr, config_len) };
    let _config_str = unsafe { bytes_to_str(config_bytes) };

    // 简单解析：查找 auth_token 字段
    // 实际实现应该使用 JSON 解析
    // 这里简化处理，假设 token 通过 host_get_secret 获取

    unsafe {
        // 获取 bot token
        let key = b"telegram_bot_token";
        let token_len = host_get_secret(
            key.as_ptr(),
            key.len(),
            BOT_TOKEN.as_mut_ptr(),
            BOT_TOKEN.len(),
        );

        if token_len > 0 && token_len <= BOT_TOKEN.len() {
            BOT_TOKEN_LEN = token_len;
            IS_CONNECTED = true;

            // 返回成功
            let response = b"{\"status\":\"connected\",\"channel\":\"telegram\"}";
            let copy_len = response.len().min(result_max_len);
            core::ptr::copy_nonoverlapping(response.as_ptr(), result_ptr, copy_len);
            copy_len
        } else {
            // Token 获取失败
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
    message_ptr: *const u8,
    message_len: usize,
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

    // 解析消息 (简化版本)
    // 实际应该解析 JSON 获取 chat_id 和 text
    let _message_bytes = unsafe { core::slice::from_raw_parts(message_ptr, message_len) };

    // 构建 sendMessage 请求
    // 这里简化处理，实际应该解析完整的 JSON

    // 临时缓冲区用于构建 URL
    let mut url_buf = [0u8; 512];
    let method = b"sendMessage";
    let url_len = unsafe { build_api_url(&BOT_TOKEN[..BOT_TOKEN_LEN], method, &mut url_buf) };

    // 构建请求体 (简化)
    // 实际应该从 message JSON 中提取 chat_id 和 text
    let body = b"{\"chat_id\":\"123456\",\"text\":\"Hello from WASM!\"}";

    // 发送请求
    let mut response_buf = [0u8; 4096];
    let headers = b"{\"Content-Type\":\"application/json\"}";

    let response_len = unsafe {
        host_http_request(
            b"POST".as_ptr(),
            4,
            url_buf.as_ptr(),
            url_len,
            headers.as_ptr(),
            headers.len(),
            body.as_ptr(),
            body.len(),
            response_buf.as_mut_ptr(),
            response_buf.len(),
        )
    };

    // 返回响应
    let copy_len = response_len.min(result_max_len);
    unsafe {
        core::ptr::copy_nonoverlapping(response_buf.as_ptr(), result_ptr, copy_len);
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

/// 轮询 Telegram 更新
fn poll_updates() {
    unsafe {
        if !IS_CONNECTED || BOT_TOKEN_LEN == 0 {
            return;
        }
    }

    // 构建 getUpdates URL
    let mut url_buf = [0u8; 512];
    let method = b"getUpdates";
    let base_len =
        unsafe { build_api_url(unsafe { &BOT_TOKEN[..BOT_TOKEN_LEN] }, method, &mut url_buf) };

    // 添加 offset 参数
    let offset_str = unsafe { format_i64(LAST_UPDATE_ID) };
    if base_len + 8 < url_buf.len() {
        url_buf[base_len] = b'?';
        url_buf[base_len + 1..base_len + 7].copy_from_slice(b"offset=");
        // 简化：假设 offset 是个位数
        if LAST_UPDATE_ID >= 0 && LAST_UPDATE_ID < 10 {
            url_buf[base_len + 7] = b'0' + LAST_UPDATE_ID as u8;
        }
    }

    // 发送长轮询请求
    let mut response_buf = [0u8; 8192];
    let headers = b"{}";

    let response_len = unsafe {
        host_http_poll(
            url_buf.as_ptr(),
            base_len + 8,
            headers.as_ptr(),
            headers.len(),
            30000, // 30 秒超时
            response_buf.as_mut_ptr(),
            response_buf.len(),
        )
    };

    if response_len > 0 {
        // 解析响应并提取消息
        // 简化版本：直接调用 host_emit_incoming
        unsafe {
            host_emit_incoming(response_buf.as_ptr(), response_len);
        }
    }
}

/// 简单的 i64 转字符串（仅支持小数字）
fn format_i64(value: i64) -> [u8; 20] {
    let mut buf = [0u8; 20];
    let mut v = if value < 0 { -value } else { value } as u64;
    let mut pos = 19;

    if v == 0 {
        buf[pos] = b'0';
        return buf;
    }

    while v > 0 && pos > 0 {
        buf[pos] = b'0' + (v % 10) as u8;
        v /= 10;
        pos -= 1;
    }

    if value < 0 && pos > 0 {
        buf[pos] = b'-';
    }

    buf
}
