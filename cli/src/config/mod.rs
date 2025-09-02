#![allow(dead_code)]

use anyhow::Result;
use dirs::home_dir;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub mod cargo_config;
pub mod marketplace;
pub mod project;

// API endpoints
pub const MARKETPLACE_API_MAINNET: &str = "https://ic0.app";
pub const MARKETPLACE_API_LOCAL: &str = "http://localhost:4943";

// Marketplace canister IDs
pub const MARKETPLACE_CANISTER_ID_MAINNET: &str = "rdmx6-jaaaa-aaaaa-aaadq-cai"; // TODO: Deploy to mainnet
pub const MARKETPLACE_CANISTER_ID_LOCAL: &str = "crobb-r4zot-lulfi-e76ua";

// Installation URLs
pub const BRIDGE_DOWNLOAD_BASE_URL: &str = "https://icarus.dev/downloads/bridge";

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

    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;
        Ok(())
    }

    pub fn config_path() -> Result<PathBuf> {
        let home = home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        Ok(home.join(".icarus").join("config.toml"))
    }

    pub fn config_dir() -> Result<PathBuf> {
        let home = home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        Ok(home.join(".icarus"))
    }
}

pub fn get_marketplace_url(network: &str) -> &'static str {
    match network {
        "ic" => MARKETPLACE_API_MAINNET,
        "local" => MARKETPLACE_API_LOCAL,
        _ => MARKETPLACE_API_MAINNET,
    }
}

pub fn get_marketplace_canister_id(network: &str) -> &'static str {
    match network {
        "ic" => MARKETPLACE_CANISTER_ID_MAINNET,
        "local" => MARKETPLACE_CANISTER_ID_LOCAL,
        _ => MARKETPLACE_CANISTER_ID_MAINNET,
    }
}
