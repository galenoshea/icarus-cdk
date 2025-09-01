//! Cargo.toml metadata configuration for Icarus projects
//!
//! This replaces icarus.toml by using [package.metadata.icarus] in Cargo.toml

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use toml::Value;

/// Claude Desktop integration configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClaudeDesktopConfig {
    /// Automatically update Claude Desktop config on deploy
    #[serde(default)]
    pub auto_update: bool,

    /// Path to Claude Desktop config file (optional)
    #[serde(default)]
    pub config_path: String,
}

/// Icarus metadata configuration in Cargo.toml
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IcarusMetadata {
    #[serde(default)]
    pub claude_desktop: ClaudeDesktopConfig,
}

impl Default for ClaudeDesktopConfig {
    fn default() -> Self {
        Self {
            auto_update: false,
            config_path: String::new(),
        }
    }
}

impl Default for IcarusMetadata {
    fn default() -> Self {
        Self {
            claude_desktop: ClaudeDesktopConfig::default(),
        }
    }
}

/// Load Icarus configuration from Cargo.toml
pub fn load_from_cargo_toml(project_dir: &Path) -> Result<Option<IcarusMetadata>> {
    let cargo_toml_path = project_dir.join("Cargo.toml");

    if !cargo_toml_path.exists() {
        return Ok(None);
    }

    let content = std::fs::read_to_string(&cargo_toml_path)?;
    let cargo_toml: Value = toml::from_str(&content)?;

    // Look for [package.metadata.icarus]
    let metadata = cargo_toml
        .get("package")
        .and_then(|p| p.get("metadata"))
        .and_then(|m| m.get("icarus"));

    match metadata {
        Some(icarus_value) => {
            let icarus_metadata: IcarusMetadata = icarus_value.clone().try_into()?;
            Ok(Some(icarus_metadata))
        }
        None => Ok(None),
    }
}

/// Get the Claude Desktop config path, resolving relative paths
pub fn get_claude_config_path(config: &IcarusMetadata, project_dir: &Path) -> Option<PathBuf> {
    if config.claude_desktop.config_path.is_empty() {
        None
    } else {
        let path = PathBuf::from(&config.claude_desktop.config_path);
        if path.is_absolute() {
            Some(path)
        } else {
            // Resolve relative to project directory
            Some(project_dir.join(path))
        }
    }
}
