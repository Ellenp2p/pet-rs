//! Hook 点定义
//!
//! 定义所有 28 个 Hook 点及其属性。

use serde::{Deserialize, Serialize};
use std::fmt;

/// Hook 执行模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HookExecutionMode {
    /// 顺序执行（按优先级）
    Sequential,
    /// 并行执行
    Parallel,
    /// 独占执行（只有一个 Hook）
    Exclusive,
}

/// Hook 点定义
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HookPoint {
    // ===== 输入处理层 (3 个) =====
    /// 输入到达时
    OnInputReceived,
    /// 解析输入前
    BeforeInputParse,
    /// 解析输入后
    AfterInputParse,

    // ===== 上下文构建层 (4 个) =====
    /// 构建上下文前
    BeforeContextBuild,
    /// 构建上下文后
    AfterContextBuild,
    /// 加载记忆前
    BeforeMemoryLoad,
    /// 加载记忆后
    AfterMemoryLoad,

    // ===== 决策层 (4 个) =====
    /// 决策前
    BeforeDecision,
    /// 决策后
    AfterDecision,
    /// LLM 调用前
    BeforeLlmCall,
    /// LLM 调用后
    AfterLlmCall,

    // ===== 动作执行层 (4 个) =====
    /// 动作执行前
    BeforeAction,
    /// 动作执行后
    AfterAction,
    /// 工具调用前
    BeforeToolCall,
    /// 工具调用后
    AfterToolCall,

    // ===== 输出生成层 (3 个) =====
    /// 生成输出前
    BeforeOutput,
    /// 生成输出后
    AfterOutput,
    /// 发送响应前
    BeforeResponse,

    // ===== 记忆管理层 (3 个) =====
    /// 写入记忆前
    BeforeMemoryWrite,
    /// 写入记忆后
    AfterMemoryWrite,
    /// 记忆压缩前
    BeforeMemoryCompact,

    // ===== 角色/人格层 (3 个) =====
    /// 应用角色前
    BeforeRoleApply,
    /// 应用角色后
    AfterRoleApply,
    /// 人格变化时
    OnPersonalityChange,

    // ===== 生命周期层 (4 个) =====
    /// Agent 启动时
    OnAgentStart,
    /// Agent 停止时
    OnAgentStop,
    /// 会话开始时
    OnSessionStart,
    /// 会话结束时
    OnSessionEnd,
}

impl HookPoint {
    /// 获取所有 Hook 点
    pub fn all() -> Vec<HookPoint> {
        vec![
            // 输入处理层
            HookPoint::OnInputReceived,
            HookPoint::BeforeInputParse,
            HookPoint::AfterInputParse,
            // 上下文构建层
            HookPoint::BeforeContextBuild,
            HookPoint::AfterContextBuild,
            HookPoint::BeforeMemoryLoad,
            HookPoint::AfterMemoryLoad,
            // 决策层
            HookPoint::BeforeDecision,
            HookPoint::AfterDecision,
            HookPoint::BeforeLlmCall,
            HookPoint::AfterLlmCall,
            // 动作执行层
            HookPoint::BeforeAction,
            HookPoint::AfterAction,
            HookPoint::BeforeToolCall,
            HookPoint::AfterToolCall,
            // 输出生成层
            HookPoint::BeforeOutput,
            HookPoint::AfterOutput,
            HookPoint::BeforeResponse,
            // 记忆管理层
            HookPoint::BeforeMemoryWrite,
            HookPoint::AfterMemoryWrite,
            HookPoint::BeforeMemoryCompact,
            // 角色/人格层
            HookPoint::BeforeRoleApply,
            HookPoint::AfterRoleApply,
            HookPoint::OnPersonalityChange,
            // 生命周期层
            HookPoint::OnAgentStart,
            HookPoint::OnAgentStop,
            HookPoint::OnSessionStart,
            HookPoint::OnSessionEnd,
        ]
    }

    /// 获取 Hook 点的名称
    pub fn name(&self) -> &'static str {
        match self {
            // 输入处理层
            HookPoint::OnInputReceived => "on_input_received",
            HookPoint::BeforeInputParse => "before_input_parse",
            HookPoint::AfterInputParse => "after_input_parse",
            // 上下文构建层
            HookPoint::BeforeContextBuild => "before_context_build",
            HookPoint::AfterContextBuild => "after_context_build",
            HookPoint::BeforeMemoryLoad => "before_memory_load",
            HookPoint::AfterMemoryLoad => "after_memory_load",
            // 决策层
            HookPoint::BeforeDecision => "before_decision",
            HookPoint::AfterDecision => "after_decision",
            HookPoint::BeforeLlmCall => "before_llm_call",
            HookPoint::AfterLlmCall => "after_llm_call",
            // 动作执行层
            HookPoint::BeforeAction => "before_action",
            HookPoint::AfterAction => "after_action",
            HookPoint::BeforeToolCall => "before_tool_call",
            HookPoint::AfterToolCall => "after_tool_call",
            // 输出生成层
            HookPoint::BeforeOutput => "before_output",
            HookPoint::AfterOutput => "after_output",
            HookPoint::BeforeResponse => "before_response",
            // 记忆管理层
            HookPoint::BeforeMemoryWrite => "before_memory_write",
            HookPoint::AfterMemoryWrite => "after_memory_write",
            HookPoint::BeforeMemoryCompact => "before_memory_compact",
            // 角色/人格层
            HookPoint::BeforeRoleApply => "before_role_apply",
            HookPoint::AfterRoleApply => "after_role_apply",
            HookPoint::OnPersonalityChange => "on_personality_change",
            // 生命周期层
            HookPoint::OnAgentStart => "on_agent_start",
            HookPoint::OnAgentStop => "on_agent_stop",
            HookPoint::OnSessionStart => "on_session_start",
            HookPoint::OnSessionEnd => "on_session_end",
        }
    }

    /// 获取 Hook 点的执行模式
    pub fn execution_mode(&self) -> HookExecutionMode {
        match self {
            // 独占执行的 Hook
            HookPoint::BeforeDecision => HookExecutionMode::Exclusive,
            HookPoint::AfterDecision => HookExecutionMode::Exclusive,
            HookPoint::BeforeLlmCall => HookExecutionMode::Exclusive,
            HookPoint::AfterLlmCall => HookExecutionMode::Exclusive,

            // 并行执行的 Hook
            HookPoint::OnAgentStart => HookExecutionMode::Parallel,
            HookPoint::OnAgentStop => HookExecutionMode::Parallel,
            HookPoint::OnSessionStart => HookExecutionMode::Parallel,
            HookPoint::OnSessionEnd => HookExecutionMode::Parallel,

            // 其他默认顺序执行
            _ => HookExecutionMode::Sequential,
        }
    }

    /// 获取 Hook 点的描述
    pub fn description(&self) -> &'static str {
        match self {
            // 输入处理层
            HookPoint::OnInputReceived => "输入到达时",
            HookPoint::BeforeInputParse => "解析输入前",
            HookPoint::AfterInputParse => "解析输入后",
            // 上下文构建层
            HookPoint::BeforeContextBuild => "构建上下文前",
            HookPoint::AfterContextBuild => "构建上下文后",
            HookPoint::BeforeMemoryLoad => "加载记忆前",
            HookPoint::AfterMemoryLoad => "加载记忆后",
            // 决策层
            HookPoint::BeforeDecision => "决策前",
            HookPoint::AfterDecision => "决策后",
            HookPoint::BeforeLlmCall => "LLM 调用前",
            HookPoint::AfterLlmCall => "LLM 调用后",
            // 动作执行层
            HookPoint::BeforeAction => "动作执行前",
            HookPoint::AfterAction => "动作执行后",
            HookPoint::BeforeToolCall => "工具调用前",
            HookPoint::AfterToolCall => "工具调用后",
            // 输出生成层
            HookPoint::BeforeOutput => "生成输出前",
            HookPoint::AfterOutput => "生成输出后",
            HookPoint::BeforeResponse => "发送响应前",
            // 记忆管理层
            HookPoint::BeforeMemoryWrite => "写入记忆前",
            HookPoint::AfterMemoryWrite => "写入记忆后",
            HookPoint::BeforeMemoryCompact => "记忆压缩前",
            // 角色/人格层
            HookPoint::BeforeRoleApply => "应用角色前",
            HookPoint::AfterRoleApply => "应用角色后",
            HookPoint::OnPersonalityChange => "人格变化时",
            // 生命周期层
            HookPoint::OnAgentStart => "Agent 启动时",
            HookPoint::OnAgentStop => "Agent 停止时",
            HookPoint::OnSessionStart => "会话开始时",
            HookPoint::OnSessionEnd => "会话结束时",
        }
    }

    /// 从名称解析 Hook 点
    pub fn from_name(name: &str) -> Option<HookPoint> {
        match name {
            // 输入处理层
            "on_input_received" => Some(HookPoint::OnInputReceived),
            "before_input_parse" => Some(HookPoint::BeforeInputParse),
            "after_input_parse" => Some(HookPoint::AfterInputParse),
            // 上下文构建层
            "before_context_build" => Some(HookPoint::BeforeContextBuild),
            "after_context_build" => Some(HookPoint::AfterContextBuild),
            "before_memory_load" => Some(HookPoint::BeforeMemoryLoad),
            "after_memory_load" => Some(HookPoint::AfterMemoryLoad),
            // 决策层
            "before_decision" => Some(HookPoint::BeforeDecision),
            "after_decision" => Some(HookPoint::AfterDecision),
            "before_llm_call" => Some(HookPoint::BeforeLlmCall),
            "after_llm_call" => Some(HookPoint::AfterLlmCall),
            // 动作执行层
            "before_action" => Some(HookPoint::BeforeAction),
            "after_action" => Some(HookPoint::AfterAction),
            "before_tool_call" => Some(HookPoint::BeforeToolCall),
            "after_tool_call" => Some(HookPoint::AfterToolCall),
            // 输出生成层
            "before_output" => Some(HookPoint::BeforeOutput),
            "after_output" => Some(HookPoint::AfterOutput),
            "before_response" => Some(HookPoint::BeforeResponse),
            // 记忆管理层
            "before_memory_write" => Some(HookPoint::BeforeMemoryWrite),
            "after_memory_write" => Some(HookPoint::AfterMemoryWrite),
            "before_memory_compact" => Some(HookPoint::BeforeMemoryCompact),
            // 角色/人格层
            "before_role_apply" => Some(HookPoint::BeforeRoleApply),
            "after_role_apply" => Some(HookPoint::AfterRoleApply),
            "on_personality_change" => Some(HookPoint::OnPersonalityChange),
            // 生命周期层
            "on_agent_start" => Some(HookPoint::OnAgentStart),
            "on_agent_stop" => Some(HookPoint::OnAgentStop),
            "on_session_start" => Some(HookPoint::OnSessionStart),
            "on_session_end" => Some(HookPoint::OnSessionEnd),
            _ => None,
        }
    }
}

impl fmt::Display for HookPoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_point_all() {
        let all_hooks = HookPoint::all();
        assert_eq!(all_hooks.len(), 28);
    }

    #[test]
    fn test_hook_point_name() {
        assert_eq!(HookPoint::OnInputReceived.name(), "on_input_received");
        assert_eq!(HookPoint::BeforeDecision.name(), "before_decision");
    }

    #[test]
    fn test_hook_point_from_name() {
        assert_eq!(
            HookPoint::from_name("on_input_received"),
            Some(HookPoint::OnInputReceived)
        );
        assert_eq!(HookPoint::from_name("invalid_hook"), None);
    }

    #[test]
    fn test_hook_point_execution_mode() {
        assert_eq!(
            HookPoint::BeforeDecision.execution_mode(),
            HookExecutionMode::Exclusive
        );
        assert_eq!(
            HookPoint::OnAgentStart.execution_mode(),
            HookExecutionMode::Parallel
        );
        assert_eq!(
            HookPoint::OnInputReceived.execution_mode(),
            HookExecutionMode::Sequential
        );
    }
}
