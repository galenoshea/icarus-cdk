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

/// ChatGPT Desktop integration configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatGptDesktopConfig {
    /// Automatically update ChatGPT Desktop config on deploy
    #[serde(default)]
    pub auto_update: bool,

    /// Path to ChatGPT Desktop config file (optional)
    #[serde(default)]
    pub config_path: String,
}

/// Claude Code integration configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClaudeCodeConfig {
    /// Automatically update Claude Code config on deploy
    #[serde(default)]
    pub auto_update: bool,

    /// Path to Claude Code config file (optional)
    #[serde(default)]
    pub config_path: String,
}

/// Icarus metadata configuration in Cargo.toml
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IcarusMetadata {
    #[serde(default)]
    pub claude_desktop: ClaudeDesktopConfig,

    #[serde(default)]
    pub chatgpt_desktop: ChatGptDesktopConfig,

    #[serde(default)]
    pub claude_code: ClaudeCodeConfig,
}

impl Default for ClaudeDesktopConfig {
    fn default() -> Self {
        Self {
            auto_update: false,
            config_path: String::new(),
        }
    }
}

impl Default for ChatGptDesktopConfig {
    fn default() -> Self {
        Self {
            auto_update: false,
            config_path: String::new(),
        }
    }
}

impl Default for ClaudeCodeConfig {
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
            chatgpt_desktop: ChatGptDesktopConfig::default(),
            claude_code: ClaudeCodeConfig::default(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_default_configs() {
        let claude_config = ClaudeDesktopConfig::default();
        assert!(!claude_config.auto_update);
        assert!(claude_config.config_path.is_empty());

        let chatgpt_config = ChatGptDesktopConfig::default();
        assert!(!chatgpt_config.auto_update);
        assert!(chatgpt_config.config_path.is_empty());

        let claude_code_config = ClaudeCodeConfig::default();
        assert!(!claude_code_config.auto_update);
        assert!(claude_code_config.config_path.is_empty());

        let icarus_metadata = IcarusMetadata::default();
        assert!(!icarus_metadata.claude_desktop.auto_update);
        assert!(!icarus_metadata.chatgpt_desktop.auto_update);
        assert!(!icarus_metadata.claude_code.auto_update);
    }

    #[test]
    fn test_load_from_cargo_toml_with_no_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_toml_path = temp_dir.path().join("Cargo.toml");

        // Create Cargo.toml without metadata.icarus section
        let cargo_toml_content = r#"
[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
icarus = "0.1.0"
"#;
        fs::write(&cargo_toml_path, cargo_toml_content).unwrap();

        let result = load_from_cargo_toml(temp_dir.path()).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_load_from_cargo_toml_with_no_cargo_toml() {
        let temp_dir = TempDir::new().unwrap();

        let result = load_from_cargo_toml(temp_dir.path()).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_load_from_cargo_toml_with_basic_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_toml_path = temp_dir.path().join("Cargo.toml");

        // Create Cargo.toml with basic metadata.icarus section
        let cargo_toml_content = r#"
[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[package.metadata.icarus]
claude_desktop.auto_update = true

[dependencies]
icarus = "0.1.0"
"#;
        fs::write(&cargo_toml_path, cargo_toml_content).unwrap();

        let result = load_from_cargo_toml(temp_dir.path()).unwrap();
        assert!(result.is_some());

        let metadata = result.unwrap();
        assert!(metadata.claude_desktop.auto_update);
        assert!(!metadata.chatgpt_desktop.auto_update); // Default
        assert!(!metadata.claude_code.auto_update); // Default
    }

    #[test]
    fn test_load_from_cargo_toml_with_full_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_toml_path = temp_dir.path().join("Cargo.toml");

        // Create Cargo.toml with full metadata.icarus section
        let cargo_toml_content = r#"
[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[package.metadata.icarus]
claude_desktop.auto_update = true
claude_desktop.config_path = "/custom/claude/config.json"
chatgpt_desktop.auto_update = true
chatgpt_desktop.config_path = "/custom/chatgpt/config.json"
claude_code.auto_update = false
claude_code.config_path = "/custom/claude-code/config.json"

[dependencies]
icarus = "0.1.0"
"#;
        fs::write(&cargo_toml_path, cargo_toml_content).unwrap();

        let result = load_from_cargo_toml(temp_dir.path()).unwrap();
        assert!(result.is_some());

        let metadata = result.unwrap();

        // Claude Desktop config
        assert!(metadata.claude_desktop.auto_update);
        assert_eq!(
            metadata.claude_desktop.config_path,
            "/custom/claude/config.json"
        );

        // ChatGPT Desktop config
        assert!(metadata.chatgpt_desktop.auto_update);
        assert_eq!(
            metadata.chatgpt_desktop.config_path,
            "/custom/chatgpt/config.json"
        );

        // Claude Code config
        assert!(!metadata.claude_code.auto_update);
        assert_eq!(
            metadata.claude_code.config_path,
            "/custom/claude-code/config.json"
        );
    }

    #[test]
    fn test_load_from_cargo_toml_with_selective_clients() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_toml_path = temp_dir.path().join("Cargo.toml");

        // Create Cargo.toml with selective client configuration
        let cargo_toml_content = r#"
[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[package.metadata.icarus]
claude_desktop.auto_update = true
chatgpt_desktop.auto_update = false
claude_code.auto_update = true

[dependencies]
icarus = "0.1.0"
"#;
        fs::write(&cargo_toml_path, cargo_toml_content).unwrap();

        let result = load_from_cargo_toml(temp_dir.path()).unwrap();
        assert!(result.is_some());

        let metadata = result.unwrap();
        assert!(metadata.claude_desktop.auto_update);
        assert!(!metadata.chatgpt_desktop.auto_update);
        assert!(metadata.claude_code.auto_update);
    }

    #[test]
    fn test_load_from_cargo_toml_with_all_clients_disabled() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_toml_path = temp_dir.path().join("Cargo.toml");

        // Create Cargo.toml with all clients disabled
        let cargo_toml_content = r#"
[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[package.metadata.icarus]
claude_desktop.auto_update = false
chatgpt_desktop.auto_update = false
claude_code.auto_update = false

[dependencies]
icarus = "0.1.0"
"#;
        fs::write(&cargo_toml_path, cargo_toml_content).unwrap();

        let result = load_from_cargo_toml(temp_dir.path()).unwrap();
        assert!(result.is_some());

        let metadata = result.unwrap();
        assert!(!metadata.claude_desktop.auto_update);
        assert!(!metadata.chatgpt_desktop.auto_update);
        assert!(!metadata.claude_code.auto_update);
    }

    #[test]
    fn test_get_claude_config_path_with_no_path() {
        let metadata = IcarusMetadata::default();
        let temp_dir = TempDir::new().unwrap();

        let result = get_claude_config_path(&metadata, temp_dir.path());
        assert!(result.is_none());
    }

    #[test]
    fn test_get_claude_config_path_with_absolute_path() {
        let mut metadata = IcarusMetadata::default();
        metadata.claude_desktop.config_path = "/absolute/path/config.json".to_string();
        let temp_dir = TempDir::new().unwrap();

        let result = get_claude_config_path(&metadata, temp_dir.path()).unwrap();
        assert_eq!(result, PathBuf::from("/absolute/path/config.json"));
    }

    #[test]
    fn test_get_claude_config_path_with_relative_path() {
        let mut metadata = IcarusMetadata::default();
        metadata.claude_desktop.config_path = "relative/config.json".to_string();
        let temp_dir = TempDir::new().unwrap();

        let result = get_claude_config_path(&metadata, temp_dir.path()).unwrap();
        let expected = temp_dir.path().join("relative/config.json");
        assert_eq!(result, expected);
    }

    #[test]
    fn test_load_from_cargo_toml_with_malformed_toml() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_toml_path = temp_dir.path().join("Cargo.toml");

        // Create malformed Cargo.toml
        let cargo_toml_content = r#"
[package
name = "test-project"
version = "0.1.0"
"#;
        fs::write(&cargo_toml_path, cargo_toml_content).unwrap();

        let result = load_from_cargo_toml(temp_dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_load_from_cargo_toml_with_invalid_metadata_structure() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_toml_path = temp_dir.path().join("Cargo.toml");

        // Create Cargo.toml with invalid metadata structure
        let cargo_toml_content = r#"
[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[package.metadata.icarus]
claude_desktop = "invalid_value_not_table"

[dependencies]
icarus = "0.1.0"
"#;
        fs::write(&cargo_toml_path, cargo_toml_content).unwrap();

        let result = load_from_cargo_toml(temp_dir.path());
        assert!(result.is_err());
    }
}
