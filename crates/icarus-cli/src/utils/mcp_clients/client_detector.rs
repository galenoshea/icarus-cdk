//! Client detection utilities for finding installed AI clients

use anyhow::Result;
use std::path::PathBuf;

/// Get the application support directory for the current platform
pub fn get_app_support_dir() -> Result<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        dirs::home_dir()
            .map(|h| h.join("Library/Application Support"))
            .ok_or_else(|| anyhow::anyhow!("Could not find Application Support directory"))
    }
    #[cfg(target_os = "windows")]
    {
        std::env::var("APPDATA")
            .map(PathBuf::from)
            .map_err(|_| anyhow::anyhow!("Could not find APPDATA directory"))
    }
    #[cfg(target_os = "linux")]
    {
        dirs::config_dir().ok_or_else(|| anyhow::anyhow!("Could not find config directory"))
    }
}

/// Get the VS Code extensions directory for the current platform
pub fn get_vscode_extensions_dir() -> Result<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        dirs::home_dir()
            .map(|h| h.join("Library/Application Support/Code/User/globalStorage"))
            .ok_or_else(|| anyhow::anyhow!("Could not find VS Code extensions directory"))
    }
    #[cfg(target_os = "windows")]
    {
        std::env::var("APPDATA")
            .map(|appdata| PathBuf::from(appdata).join("Code/User/globalStorage"))
            .map_err(|_| anyhow::anyhow!("Could not find VS Code extensions directory"))
    }
    #[cfg(target_os = "linux")]
    {
        dirs::config_dir()
            .map(|c| c.join("Code/User/globalStorage"))
            .ok_or_else(|| anyhow::anyhow!("Could not find VS Code extensions directory"))
    }
}

/// Check if an application is installed by looking for it in common locations
pub fn check_app_installed(app_name: &str) -> bool {
    // Check if the app is in PATH
    if which::which(app_name).is_ok() {
        return true;
    }

    // Check platform-specific application directories
    #[cfg(target_os = "macos")]
    {
        let app_path = PathBuf::from(format!("/Applications/{}.app", app_name));
        if app_path.exists() {
            return true;
        }
    }

    #[cfg(target_os = "windows")]
    {
        // Check Program Files directories
        let program_files = vec![
            std::env::var("ProgramFiles").unwrap_or_default(),
            std::env::var("ProgramFiles(x86)").unwrap_or_default(),
        ];

        for pf in program_files {
            if !pf.is_empty() {
                let app_path = PathBuf::from(pf).join(app_name);
                if app_path.exists() {
                    return true;
                }
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        // Check common installation directories
        let common_dirs = vec!["/usr/bin", "/usr/local/bin", "/opt", "/snap/bin"];

        for dir in common_dirs {
            let app_path = PathBuf::from(dir).join(app_name);
            if app_path.exists() {
                return true;
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_app_support_dir() {
        let app_support_dir = get_app_support_dir();
        assert!(app_support_dir.is_ok());
    }
}
