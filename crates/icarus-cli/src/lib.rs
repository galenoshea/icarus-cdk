//! Icarus CLI Library
//!
//! This library provides a minimal public API for the Icarus CLI.
//! Use the re-exported types and functions for the supported public interface.
//!
//! Internal modules are available but hidden from documentation.
//! Only the re-exported items below constitute the stable public API.

#[doc(hidden)]
pub mod config;
#[doc(hidden)]
pub mod templates;
#[doc(hidden)]
pub mod utils;

// Public domain types module
pub mod types;

// Re-export commonly used types for convenience
pub use config::mcp::{McpConfig, McpConfigMetadata, McpConfigStats, McpServerConfig};

// Re-export domain types
pub use types::{CanisterId, Network, ServerName};

// Re-export utility functions used by tests and library consumers
pub use utils::{
    client_detector::{
        detect_installed_clients, get_all_client_configs, get_chatgpt_desktop_config_path,
        get_claude_code_config_path, get_claude_desktop_config_path, get_continue_config_path,
    },
    project::{
        create_project_structure, find_project_root, is_icarus_project, load_project_config,
        validate_project_structure, ProjectConfig,
    },
};
