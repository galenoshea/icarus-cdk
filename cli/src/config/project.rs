//! Project configuration parsing for icarus.toml

use serde::{Deserialize, Serialize};

/// Main project configuration structure
#[derive(Debug, Serialize, Deserialize)]
pub struct IcarusProjectConfig {
    pub project: ProjectInfo,
    pub claude_desktop: ClaudeDesktopConfig,
}

/// Project metadata
#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub name: String,
    pub version: String,
}

/// Claude Desktop integration configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct ClaudeDesktopConfig {
    /// Path to Claude Desktop config file (optional)
    #[serde(default)]
    pub config_path: String,

    /// Automatically update Claude Desktop config on deploy
    #[serde(default)]
    pub auto_update: bool,
}
