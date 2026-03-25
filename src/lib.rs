#![allow(clippy::type_complexity)]

// Core modules
pub mod config;
pub mod dependency;
pub mod error;
pub mod hooks;
pub mod network;
pub mod permission;

// Agent modules
pub mod agent;
pub mod communication;
pub mod context;
pub mod decision;
pub mod memory;

// Plugin modules
pub mod plugins;

// Legacy modules
pub mod components;
pub mod events;
pub mod systems;

#[cfg(feature = "wasm-plugin")]
pub mod wasm;

pub mod prelude;
