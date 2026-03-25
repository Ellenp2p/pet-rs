//! Agent 主循环
//!
//! 定义 Agent 的主执行循环。

use super::core::Agent;
use crate::error::FrameworkError;
use crate::hooks::HookPoint;

/// Agent 主循环
pub struct AgentLoop {
    /// Agent
    agent: Agent,
}

impl AgentLoop {
    /// 创建新的 Agent 主循环
    pub fn new(agent: Agent) -> Self {
        Self { agent }
    }

    /// 获取 Agent
    pub fn agent(&self) -> &Agent {
        &self.agent
    }

    /// 获取可变 Agent
    pub fn agent_mut(&mut self) -> &mut Agent {
        &mut self.agent
    }

    /// 处理输入
    pub fn process_input(&mut self, input: String) -> Result<String, FrameworkError> {
        // 触发 on_input_received hook
        let context =
            crate::hooks::HookContext::new(HookPoint::OnInputReceived, self.agent.id().to_string())
                .with_input(serde_json::json!({"message": input.clone()}));

        let result = self.agent.hooks().trigger("on_input_received", &context)?;

        if result.is_blocked() {
            return Err(FrameworkError::Other("Input blocked by hook".to_string()));
        }

        // 这里可以添加更多的处理逻辑
        // 例如：解析输入、构建上下文、做出决策等

        Ok(format!("Processed: {}", input))
    }

    /// 启动循环
    pub fn start(&mut self) -> Result<(), FrameworkError> {
        self.agent.start()
    }

    /// 停止循环
    pub fn stop(&mut self) -> Result<(), FrameworkError> {
        self.agent.stop()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::core::AgentConfig;

    #[test]
    fn test_agent_loop_creation() {
        let config = AgentConfig::default();
        let agent = Agent::new(config).unwrap();
        let agent_loop = AgentLoop::new(agent);
        assert_eq!(agent_loop.agent().name(), "Agent");
    }
}
