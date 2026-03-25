// Core modules
pub use crate::config::{ConfigValue, PluginConfigManager};
pub use crate::error::FrameworkError;
pub use crate::hooks::context::HookResult;
pub use crate::hooks::{HookContext, HookPoint, HookRegistry, HookRunner};
pub use crate::network::{NetworkChannel, NetworkConfig};

// Agent modules
pub use crate::agent::core::{Agent, AgentConfig, AgentState};
pub use crate::agent::loop_impl::AgentLoop;
pub use crate::agent::personality::{Personality, PersonalityTrait};
pub use crate::agent::role::{Role, RoleTrait};

// Hook modules
pub use crate::hooks::points::HookExecutionMode;
pub use crate::hooks::registry::HookRegistration;

// Decision modules
pub use crate::decision::engine::{
    Decision, DecisionContext, DecisionEngine, DecisionEngineTrait, DecisionEngineType,
    DecisionType,
};
pub use crate::decision::rule_based::{Rule, RuleBasedEngine};

// Memory modules
pub use crate::memory::long_term::LongTermMemory;
pub use crate::memory::memory_impl::{Memory, MemoryEntry};
pub use crate::memory::short_term::ShortTermMemory;
pub use crate::memory::working::WorkingMemory;

// Context modules
pub use crate::context::builder::ContextBuilder;
pub use crate::context::context_impl::{Context, HistoryEntry};
pub use crate::context::window::ContextWindow;

// Communication modules
pub use crate::communication::channel::{CLIChannel, Channel, ChannelTrait, ChannelType};
pub use crate::communication::message::{Message, MessageType};
pub use crate::communication::router::{MessageHandler, MessageRouter};

// WASM modules
#[cfg(feature = "wasm-plugin")]
pub use crate::wasm::WasmEntityId;

#[cfg(feature = "wasm-plugin")]
pub use crate::wasm::WasmPluginHost;
