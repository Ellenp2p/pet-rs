//! 插件系统模块
//!
//! 提供插件加载、发现、验证、生命周期管理等功能。

pub mod capabilities;
pub mod discovery;
pub mod lifecycle;
pub mod loader;
pub mod manifest;
pub mod slots;
pub mod validator;

pub use capabilities::{Capability, CapabilityRegistry};
pub use discovery::PluginDiscovery;
pub use lifecycle::PluginLifecycleManager;
pub use loader::PluginLoader;
pub use manifest::PluginManifestLoader;
pub use slots::{Slot, SlotManager};
pub use validator::PluginValidator;
