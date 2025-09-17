//! Extension modules for Icarus canisters
//!
//! This module provides pre-built extensions for common canister initialization needs.

pub mod wasi;

pub use wasi::{WasiConfig, WasiExtension};
