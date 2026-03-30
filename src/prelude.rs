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
pub use crate::memory::compaction::{CompactionStrategy, MemoryCompactor, MemoryPersistence};
pub use crate::memory::long_term::LongTermMemory;
pub use crate::memory::memory_impl::{Memory, MemoryEntry};
pub use crate::memory::short_term::ShortTermMemory;
pub use crate::memory::working::WorkingMemory;

// Context modules
pub use crate::context::builder::ContextBuilder;
pub use crate::context::context_impl::{Context, HistoryEntry};
pub use crate::context::window::ContextWindow;

// Communication modules (legacy)
pub use crate::communication::channel::{CLIChannel, Channel, ChannelTrait};
pub use crate::communication::message::{Message, MessageType};
pub use crate::communication::router::MessageRouter;

// Channel modules (new - WASM plugin channels)
#[cfg(feature = "wasm-plugin")]
pub use crate::channels::ChannelPluginManager;
#[cfg(feature = "wasm-plugin")]
pub use crate::channels::WasmChannelPlugin;
pub use crate::channels::{
    ChannelAdapter, ChannelConfig, ChannelError, ChannelMessage, ChannelStatus, ChannelType,
    MediaType, MessageContent, MessageHandler, OutboundMessage, SenderInfo, SessionId,
};

// Session modules
pub use crate::session::{Session, SessionConfig, SessionManager, SessionPermissions, SessionType};

// Plugin modules
pub use crate::plugins::capabilities::{Capability, CapabilityProvider, CapabilityRegistry};
pub use crate::plugins::discovery::{DiscoveredPlugin, PluginDiscovery};
pub use crate::plugins::lifecycle::{LifecycleEvent, LifecycleHook, PluginLifecycleManager};
pub use crate::plugins::loader::PluginLoader;
pub use crate::plugins::manifest::PluginManifestLoader;
pub use crate::plugins::slots::{Slot, SlotManager, SlotRegistration};
pub use crate::plugins::validator::{PluginValidator, ValidationResult};

// AI modules
pub use crate::ai::adapters::{AnthropicProvider, OpenAIProvider};
pub use crate::ai::budget::{BudgetConfig, BudgetStatus, BudgetTracker};
pub use crate::ai::manager::AIConfig;
pub use crate::ai::pricing::PricingTable;
pub use crate::ai::rate_limiter::RateLimiter;
pub use crate::ai::usage::{UsageStats, UsageTracker};
pub use crate::ai::{
    AIError, AIProvider, AIProviderManager, ChatMessage, ChatResponse, ProviderConfig,
    ProviderType, TokenUsage,
};

// WASM modules
#[cfg(feature = "wasm-plugin")]
pub use crate::wasm::WasmEntityId;

#[cfg(feature = "wasm-plugin")]
pub use crate::wasm::WasmPluginHost;

// WASM ABI modules
#[cfg(feature = "wasm-plugin")]
pub use crate::wasm::abi::{
    ConfigFieldType, ConfigSchema, HookCallContext, HookCallResult, PluginDependency,
    PluginManifest, PluginPermission, PluginType,
};
