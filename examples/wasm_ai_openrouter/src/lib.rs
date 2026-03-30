//! OpenRouter AI Provider WASM Plugin
//!
//! 功能：
//! 1. 支持 OpenRouter API（100+ 模型）
//! 2. 自带计费计算
//! 3. 自带速率限制
//! 4. 通过宿主函数进行 HTTP 请求
//!
//! ## 构建
//!
//! ```bash
//! cd examples/wasm_ai_openrouter
//! cargo build --target wasm32-unknown-unknown --release
//! ```
//!
//! ## 导出函数
//!
//! - `wasm_plugin_name` - 插件名称
//! - `wasm_plugin_version` - 插件版本
//! - `wasm_plugin_on_tick` - 定时回调
//! - `wasm_plugin_on_event` - 事件处理
//! - `wasm_ai_provider_models` - 支持的模型列表
//! - `wasm_ai_chat` - 聊天请求
//! - `wasm_ai_calculate_cost` - 计算费用
//! - `wasm_ai_check_rate_limit` - 检查速率限制

#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// ========== 插件元数据 ==========

const PLUGIN_NAME: &[u8] = b"OpenRouter\0";

#[no_mangle]
pub extern "C" fn wasm_plugin_name() -> *const u8 {
    PLUGIN_NAME.as_ptr()
}

#[no_mangle]
pub extern "C" fn wasm_plugin_name_len() -> usize {
    PLUGIN_NAME.len() - 1
}

const PLUGIN_VERSION: &[u8] = b"1.0.0\0";

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
    // 基础宿主函数
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

    // AI 专用宿主函数
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

    fn host_record_usage(
        provider_ptr: *const u8,
        provider_len: usize,
        model_ptr: *const u8,
        model_len: usize,
        input_tokens: u32,
        output_tokens: u32,
        cost_bytes_ptr: *const u8,
    );

    fn host_check_budget() -> u32;

    fn host_emit_chunk(callback_id: u32, chunk_ptr: *const u8, chunk_len: usize);
}

// ========== 状态管理 ==========

struct RateLimitState {
    requests_this_minute: u32,
    requests_this_hour: u32,
    tokens_this_minute: u32,
    tokens_this_hour: u32,
    last_reset_minute: u64,
    last_reset_hour: u64,
}

impl RateLimitState {
    const fn new() -> Self {
        Self {
            requests_this_minute: 0,
            requests_this_hour: 0,
            tokens_this_minute: 0,
            tokens_this_hour: 0,
            last_reset_minute: 0,
            last_reset_hour: 0,
        }
    }

    fn check_and_record(&mut self, tokens: u32, current_time: u64) -> bool {
        // 每分钟重置
        if current_time - self.last_reset_minute >= 60 {
            self.requests_this_minute = 0;
            self.tokens_this_minute = 0;
            self.last_reset_minute = current_time;
        }

        // 每小时重置
        if current_time - self.last_reset_hour >= 3600 {
            self.requests_this_hour = 0;
            self.tokens_this_hour = 0;
            self.last_reset_hour = current_time;
        }

        // 检查限制 (20 req/min, 200 req/hour, 40k tokens/min, 400k tokens/hour)
        if self.requests_this_minute >= 20 {
            return false;
        }
        if self.requests_this_hour >= 200 {
            return false;
        }
        if self.tokens_this_minute + tokens >= 40000 {
            return false;
        }
        if self.tokens_this_hour + tokens >= 400000 {
            return false;
        }

        // 记录
        self.requests_this_minute += 1;
        self.requests_this_hour += 1;
        self.tokens_this_minute += tokens;
        self.tokens_this_hour += tokens;

        true
    }
}

// 价格表 (per 1M tokens)
struct Pricing {
    input_price: u32, // 以 0.01 美分为单位
    output_price: u32,
}

fn get_pricing(model: &[u8]) -> Pricing {
    // 简单的模型匹配
    let model_str = unsafe { core::str::from_utf8_unchecked(model) };

    if model_str.contains("free") {
        Pricing {
            input_price: 0,
            output_price: 0,
        }
    } else if model_str.contains("llama-3-70b") {
        Pricing {
            input_price: 50,
            output_price: 50,
        } // $0.50/1M
    } else if model_str.contains("claude") {
        Pricing {
            input_price: 250,
            output_price: 1250,
        } // $2.50/$12.50
    } else if model_str.contains("gpt-4") {
        Pricing {
            input_price: 250,
            output_price: 1000,
        } // $2.50/$10.00
    } else {
        Pricing {
            input_price: 10,
            output_price: 10,
        } // 默认 $0.10/1M
    }
}

// 全局状态
static mut RATE_LIMIT_STATE: RateLimitState = RateLimitState::new();
static mut TOTAL_REQUESTS: u64 = 0;
static mut TOTAL_INPUT_TOKENS: u64 = 0;
static mut TOTAL_OUTPUT_TOKENS: u64 = 0;
static mut TOTAL_COST: u64 = 0; // 以 0.01 美分为单位

// ========== 导出函数 ==========

#[no_mangle]
pub extern "C" fn wasm_plugin_on_tick(_entity_id: u64) {
    // 定期导出状态
    unsafe {
        let key = b"total_requests";
        let value = TOTAL_REQUESTS.to_le_bytes();
        wasm_plugin_set_data(key.as_ptr(), key.len(), value.as_ptr(), value.len());

        let key = b"total_input_tokens";
        let value = TOTAL_INPUT_TOKENS.to_le_bytes();
        wasm_plugin_set_data(key.as_ptr(), key.len(), value.as_ptr(), value.len());

        let key = b"total_output_tokens";
        let value = TOTAL_OUTPUT_TOKENS.to_le_bytes();
        wasm_plugin_set_data(key.as_ptr(), key.len(), value.as_ptr(), value.len());

        let key = b"total_cost_cents";
        let value = TOTAL_COST.to_le_bytes();
        wasm_plugin_set_data(key.as_ptr(), key.len(), value.as_ptr(), value.len());
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
    let event_str = unsafe {
        core::str::from_utf8(core::slice::from_raw_parts(event_ptr, event_len)).unwrap_or("")
    };

    match event_str {
        "chat" => {
            // 处理聊天请求
            // 实际实现中，这里会解析 data 中的请求 JSON，
            // 调用 host_http_request 发送请求，
            // 然后解析响应并返回

            // 示例：更新统计
            unsafe {
                TOTAL_REQUESTS += 1;
            }
        }
        "check_rate_limit" => {
            // 导出速率限制状态
            unsafe {
                let key = b"rate_limit_ok";
                let value = [1u8]; // 假设正常
                wasm_plugin_set_data(key.as_ptr(), key.len(), value.as_ptr(), value.len());
            }
        }
        _ => {}
    }

    let _ = (entity_id, data_ptr, data_len);
}

#[no_mangle]
pub extern "C" fn wasm_plugin_on_load() {
    // 初始化
    unsafe {
        RATE_LIMIT_STATE = RateLimitState::new();
        TOTAL_REQUESTS = 0;
        TOTAL_INPUT_TOKENS = 0;
        TOTAL_OUTPUT_TOKENS = 0;
        TOTAL_COST = 0;
    }
}

#[no_mangle]
pub extern "C" fn wasm_plugin_on_unload() {
    // 清理
}

#[no_mangle]
pub extern "C" fn wasm_plugin_on_error(_error_code: u32) {
    // 错误处理
}

// ========== AI 专用导出 ==========

/// 获取支持的模型列表（JSON 格式）
const SUPPORTED_MODELS: &[u8] = b"[\"stepfun/step-3.5-flash:free\",\"meta-llama/llama-3-70b-instruct\",\"anthropic/claude-3-haiku\",\"openai/gpt-4o-mini\",\"google/gemma-2-9b-it:free\"]\0";

#[no_mangle]
pub extern "C" fn wasm_ai_provider_models() -> *const u8 {
    SUPPORTED_MODELS.as_ptr()
}

#[no_mangle]
pub extern "C" fn wasm_ai_provider_models_len() -> usize {
    SUPPORTED_MODELS.len() - 1
}

/// 检查速率限制
/// 返回: 0 = 可用, 1 = 受限
#[no_mangle]
pub extern "C" fn wasm_ai_check_rate_limit(estimated_tokens: u32) -> u32 {
    // 简化实现：使用实体 ID 作为时间戳
    let current_time = 0; // 实际应该从宿主获取

    unsafe {
        if RATE_LIMIT_STATE.check_and_record(estimated_tokens, current_time) {
            0 // 可用
        } else {
            1 // 受限
        }
    }
}

/// 计算费用
/// 返回: 费用（以 0.01 美分为单位）
#[no_mangle]
pub extern "C" fn wasm_ai_calculate_cost(
    model_ptr: *const u8,
    model_len: usize,
    input_tokens: u32,
    output_tokens: u32,
) -> u64 {
    let model = unsafe { core::slice::from_raw_parts(model_ptr, model_len) };
    let pricing = get_pricing(model);

    // 计算费用 (tokens / 1M * price)
    let input_cost = (input_tokens as u64 * pricing.input_price as u64) / 1_000_000;
    let output_cost = (output_tokens as u64 * pricing.output_price as u64) / 1_000_000;

    input_cost + output_cost
}

/// 聊天请求
/// 参数: request_json (包含 messages, model, max_tokens, temperature)
/// 返回: response_json (包含 content, usage, model, finish_reason)
///
/// 注意: 这个函数是同步的，实际的 HTTP 请求通过 host_http_request 宿主函数进行
#[no_mangle]
pub extern "C" fn wasm_ai_chat(
    request_ptr: *const u8,
    request_len: usize,
    result_ptr: *mut u8,
    result_max_len: usize,
) -> usize {
    // 检查预算
    let budget_status = unsafe { host_check_budget() };
    if budget_status == 1 {
        // 预算超限
        return 0;
    }

    // 这里应该:
    // 1. 解析 request_json
    // 2. 读取 API Key (通过 host_get_secret)
    // 3. 构建 OpenRouter API 请求
    // 4. 调用 host_http_request 发送请求
    // 5. 解析响应
    // 6. 计算费用
    // 7. 调用 host_record_usage 记录使用量
    // 8. 返回 response_json

    // 简化示例：返回错误信息
    let error_msg = b"{\"error\":\"WASM AI chat not fully implemented - needs host_http_request\"}";
    let copy_len = error_msg.len().min(result_max_len);
    unsafe {
        core::ptr::copy_nonoverlapping(error_msg.as_ptr(), result_ptr, copy_len);
    }
    copy_len
}

/// 流式聊天请求
/// 参数: request_json, callback_id
/// 通过 host_emit_chunk 返回片段
#[no_mangle]
pub extern "C" fn wasm_ai_chat_stream(
    request_ptr: *const u8,
    request_len: usize,
    callback_id: u32,
) {
    // 类似 wasm_ai_chat，但使用流式响应
    // 需要解析 SSE 响应并通过 host_emit_chunk 发送片段

    let error_msg = b"{\"error\":\"WASM AI stream chat not fully implemented\"}";
    unsafe {
        host_emit_chunk(callback_id, error_msg.as_ptr(), error_msg.len());
    }

    let _ = (request_ptr, request_len);
}
