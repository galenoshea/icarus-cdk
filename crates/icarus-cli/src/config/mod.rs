use anyhow::Result;
use dirs::home_dir;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub mod cargo_config;
pub mod project;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IcarusConfig {
    pub session_token: Option<String>,
    pub device_id: Option<String>,
    pub last_update_check: Option<u64>,
    pub telemetry_enabled: bool,
}

impl Default for IcarusConfig {
    fn default() -> Self {
        Self {
            session_token: None,
            device_id: None,
            last_update_check: None,
            telemetry_enabled: true,
        }
    }
}

impl IcarusConfig {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            Ok(toml::from_str(&content)?)
        } else {
            Ok(Self::default())
        }
    }

    pub fn config_path() -> Result<PathBuf> {
        let home = home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        Ok(home.join(".icarus").join("config.toml"))
    }
}
