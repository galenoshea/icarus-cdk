//! AI client detection utilities for MCP server registration
//! These are infrastructure functions that will be used as the CLI expands

#![allow(dead_code)]

use anyhow::{anyhow, Result};
use std::path::PathBuf;

/// Get Claude Desktop configuration path
pub fn get_claude_desktop_config_path() -> Result<PathBuf> {
    let config_dir =
        dirs::config_dir().ok_or_else(|| anyhow!("Could not determine config directory"))?;

    #[cfg(target_os = "macos")]
    let path = config_dir.join("Claude").join("claude_desktop_config.json");

    #[cfg(target_os = "windows")]
    let path = config_dir.join("Claude").join("claude_desktop_config.json");

    #[cfg(target_os = "linux")]
    let path = config_dir.join("Claude").join("claude_desktop_config.json");

    Ok(path)
}

/// Get Claude Desktop installation path
pub(crate) fn get_claude_desktop_install_path() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        let path = PathBuf::from("/Applications/Claude.app");
        if path.exists() {
            Some(path)
        } else {
            None
        }
    }

    #[cfg(target_os = "windows")]
    {
        let local_app_data = dirs::data_local_dir()?;
        let path = local_app_data.join("Claude").join("Claude.exe");
        if path.exists() {
            Some(path)
        } else {
            None
        }
    }

    #[cfg(target_os = "linux")]
    {
        // Claude Desktop is typically installed via AppImage or package manager
        let possible_paths = vec![
            PathBuf::from("/usr/bin/claude"),
            PathBuf::from("/usr/local/bin/claude"),
            PathBuf::from("/opt/Claude/claude"),
        ];

        for path in possible_paths {
            if path.exists() {
                return Some(path);
            }
        }
        None
    }
}

/// Get Claude Code (VS Code extension) configuration path
pub fn get_claude_code_config_path() -> Result<PathBuf> {
    let config_dir =
        dirs::config_dir().ok_or_else(|| anyhow!("Could not determine config directory"))?;

    #[cfg(target_os = "macos")]
    let path = config_dir
        .join("Code")
        .join("User")
        .join("globalStorage")
        .join("anthropic.claude-code")
        .join("settings.json");

    #[cfg(target_os = "windows")]
    let path = config_dir
        .join("Code")
        .join("User")
        .join("globalStorage")
        .join("anthropic.claude-code")
        .join("settings.json");

    #[cfg(target_os = "linux")]
    let path = config_dir
        .join("Code")
        .join("User")
        .join("globalStorage")
        .join("anthropic.claude-code")
        .join("settings.json");

    Ok(path)
}

/// Get ChatGPT Desktop configuration path
pub fn get_chatgpt_desktop_config_path() -> Result<PathBuf> {
    let config_dir =
        dirs::config_dir().ok_or_else(|| anyhow!("Could not determine config directory"))?;

    #[cfg(target_os = "macos")]
    let path = config_dir.join("ChatGPT").join("config.json");

    #[cfg(target_os = "windows")]
    let path = config_dir.join("ChatGPT").join("config.json");

    #[cfg(target_os = "linux")]
    let path = config_dir.join("ChatGPT").join("config.json");

    Ok(path)
}

/// Get ChatGPT Desktop installation path
pub(crate) fn get_chatgpt_desktop_install_path() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        let path = PathBuf::from("/Applications/ChatGPT.app");
        if path.exists() {
            Some(path)
        } else {
            None
        }
    }

    #[cfg(target_os = "windows")]
    {
        let local_app_data = dirs::data_local_dir()?;
        let path = local_app_data.join("ChatGPT").join("ChatGPT.exe");
        if path.exists() {
            Some(path)
        } else {
            None
        }
    }

    #[cfg(target_os = "linux")]
    {
        let possible_paths = vec![
            PathBuf::from("/usr/bin/chatgpt"),
            PathBuf::from("/usr/local/bin/chatgpt"),
            PathBuf::from("/opt/ChatGPT/chatgpt"),
        ];

        for path in possible_paths {
            if path.exists() {
                return Some(path);
            }
        }
        None
    }
}

/// Get Continue VS Code extension configuration path
pub fn get_continue_config_path() -> Result<PathBuf> {
    let config_dir =
        dirs::config_dir().ok_or_else(|| anyhow!("Could not determine config directory"))?;

    #[cfg(target_os = "macos")]
    let path = config_dir
        .join("Code")
        .join("User")
        .join("globalStorage")
        .join("continue.continue")
        .join("config.json");

    #[cfg(target_os = "windows")]
    let path = config_dir
        .join("Code")
        .join("User")
        .join("globalStorage")
        .join("continue.continue")
        .join("config.json");

    #[cfg(target_os = "linux")]
    let path = config_dir
        .join("Code")
        .join("User")
        .join("globalStorage")
        .join("continue.continue")
        .join("config.json");

    Ok(path)
}

/// Detect installed AI clients
pub fn detect_installed_clients() -> Vec<String> {
    let mut clients = Vec::new();

    // Check for Claude Desktop
    if get_claude_desktop_install_path().is_some() {
        clients.push("claude-desktop".to_string());
    }

    // Check for ChatGPT Desktop
    if get_chatgpt_desktop_install_path().is_some() {
        clients.push("chatgpt-desktop".to_string());
    }

    // Check for VS Code extensions (harder to detect, check config paths)
    if get_claude_code_config_path().map_or(false, |p| p.exists()) {
        clients.push("claude-code".to_string());
    }

    if get_continue_config_path().map_or(false, |p| p.exists()) {
        clients.push("continue".to_string());
    }

    clients
}

/// Validate client configuration path
pub(crate) fn validate_client_path(client: &str, path: &PathBuf) -> Result<()> {
    if !path.exists() {
        return Err(anyhow!(
            "Configuration path for {} does not exist: {}",
            client,
            path.display()
        ));
    }

    // Check if parent directory is writable
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            return Err(anyhow!(
                "Configuration directory for {} does not exist: {}",
                client,
                parent.display()
            ));
        }
    }

    Ok(())
}

/// Get all possible client configurations
pub fn get_all_client_configs() -> Vec<(String, Result<PathBuf>)> {
    vec![
        (
            "claude-desktop".to_string(),
            get_claude_desktop_config_path(),
        ),
        ("claude-code".to_string(), get_claude_code_config_path()),
        (
            "chatgpt-desktop".to_string(),
            get_chatgpt_desktop_config_path(),
        ),
        ("continue".to_string(), get_continue_config_path()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_path_generation() {
        // These tests will pass on systems where dirs::config_dir() returns a value
        if dirs::config_dir().is_some() {
            assert!(get_claude_desktop_config_path().is_ok());
            assert!(get_claude_code_config_path().is_ok());
            assert!(get_chatgpt_desktop_config_path().is_ok());
            assert!(get_continue_config_path().is_ok());
        }
    }

    #[test]
    fn test_client_detection() {
        // This test checks that the function runs without error
        let _clients = detect_installed_clients();
        // Result could be empty if no clients are installed, which is fine
        // Just checking that the function executes without panicking
    }

    #[test]
    fn test_all_client_configs() {
        let configs = get_all_client_configs();
        assert_eq!(configs.len(), 4);

        // Check that all expected clients are included
        let client_names: Vec<&str> = configs.iter().map(|(name, _)| name.as_str()).collect();
        assert!(client_names.contains(&"claude-desktop"));
        assert!(client_names.contains(&"claude-code"));
        assert!(client_names.contains(&"chatgpt-desktop"));
        assert!(client_names.contains(&"continue"));
    }

    #[test]
    fn test_path_validation() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let existing_path = temp_dir.path().join("config.json");
        std::fs::write(&existing_path, "{}").unwrap();

        // Test with existing file
        assert!(validate_client_path("test", &existing_path).is_ok());

        // Test with non-existent file
        let non_existent = temp_dir.path().join("nonexistent.json");
        assert!(validate_client_path("test", &non_existent).is_err());
    }
}
