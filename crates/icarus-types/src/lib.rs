//! Protocol types and utilities for bridging MCP and ICP
//! 
//! This crate provides the types and utilities needed to translate
//! between MCP's JSON-RPC protocol and ICP's canister calls.

pub mod protocol;
pub mod translator;
pub mod config;

pub use protocol::{IcarusBridgeRequest, IcarusBridgeResponse};
pub use translator::ProtocolTranslator;
pub use config::BridgeConfig;