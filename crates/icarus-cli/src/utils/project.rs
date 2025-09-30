use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs;

/// Project configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub license: Option<String>,
    pub canister_type: String,
    pub features: Vec<String>,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            name: "icarus-project".to_string(),
            version: "0.1.0".to_string(),
            description: None,
            author: None,
            license: None,
            canister_type: "rust".to_string(),
            features: vec!["default".to_string()],
        }
    }
}

/// Find the project root directory by looking for Cargo.toml or dfx.json
pub fn find_project_root() -> Result<PathBuf> {
    let mut current_dir = std::env::current_dir().context("Failed to get current directory")?;

    loop {
        // Check for Cargo.toml
        if current_dir.join("Cargo.toml").exists() {
            return Ok(current_dir);
        }

        // Check for dfx.json
        if current_dir.join("dfx.json").exists() {
            return Ok(current_dir);
        }

        // Move to parent directory
        if let Some(parent) = current_dir.parent() {
            current_dir = parent.to_path_buf();
        } else {
            break;
        }
    }

    Err(anyhow!(
        "Not in an Icarus project directory (no Cargo.toml or dfx.json found)"
    ))
}

/// Load project configuration from Cargo.toml
pub async fn load_project_config(project_root: &Path) -> Result<ProjectConfig> {
    let cargo_path = project_root.join("Cargo.toml");

    if !cargo_path.exists() {
        return Ok(ProjectConfig::default());
    }

    let content = fs::read_to_string(&cargo_path)
        .await
        .with_context(|| format!("Failed to read Cargo.toml: {}", cargo_path.display()))?;

    let cargo_toml: toml::Value =
        toml::from_str(&content).with_context(|| "Failed to parse Cargo.toml")?;

    let package = cargo_toml
        .get("package")
        .ok_or_else(|| anyhow!("No [package] section found in Cargo.toml"))?;

    let name = package
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let version = package
        .get("version")
        .and_then(|v| v.as_str())
        .unwrap_or("0.1.0")
        .to_string();

    let description = package
        .get("description")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let author = package
        .get("authors")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let license = package
        .get("license")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // Determine canister type from lib.crate-type
    let canister_type = cargo_toml
        .get("lib")
        .and_then(|lib| lib.get("crate-type"))
        .and_then(|ct| ct.as_array())
        .and_then(|arr| arr.first())
        .and_then(|v| v.as_str())
        .map(|ct| if ct == "cdylib" { "rust" } else { ct })
        .unwrap_or("rust")
        .to_string();

    // Extract features from dependencies or default features
    let mut features = vec!["default".to_string()];
    if let Some(deps) = cargo_toml.get("dependencies") {
        if deps.get("icarus").is_some() {
            features.push("mcp".to_string());
        }
        if deps.get("ic-cdk-timers").is_some() {
            features.push("timers".to_string());
        }
    }

    Ok(ProjectConfig {
        name,
        version,
        description,
        author,
        license,
        canister_type,
        features,
    })
}

/// Check if the current directory is a valid Icarus project
#[allow(dead_code)]
pub async fn is_icarus_project(path: &Path) -> bool {
    let cargo_path = path.join("Cargo.toml");
    if !cargo_path.exists() {
        return false;
    }

    // Check if Cargo.toml contains icarus dependency
    if let Ok(content) = fs::read_to_string(&cargo_path).await {
        if let Ok(cargo_toml) = toml::from_str::<toml::Value>(&content) {
            if let Some(deps) = cargo_toml.get("dependencies") {
                return deps.get("icarus").is_some();
            }
        }
    }

    false
}

/// Get project metadata including canister information
#[allow(dead_code)]
pub(crate) async fn get_project_metadata(project_root: &Path) -> Result<ProjectMetadata> {
    let config = load_project_config(project_root).await?;

    // Load dfx.json if it exists
    let dfx_config = load_dfx_config(project_root).await.ok();

    // Get canister IDs if available
    let canister_ids = load_canister_ids(project_root).await.unwrap_or_default();

    Ok(ProjectMetadata {
        config,
        dfx_config,
        canister_ids,
        project_root: project_root.to_path_buf(),
    })
}

/// Project metadata container
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct ProjectMetadata {
    pub config: ProjectConfig,
    pub dfx_config: Option<DfxConfig>,
    pub canister_ids: std::collections::HashMap<String, CanisterIds>,
    pub project_root: PathBuf,
}

/// dfx.json configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct DfxConfig {
    pub version: Option<u32>,
    pub canisters: std::collections::HashMap<String, CanisterConfig>,
    pub networks: Option<std::collections::HashMap<String, NetworkConfig>>,
}

/// Canister configuration in dfx.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CanisterConfig {
    #[serde(rename = "type")]
    pub canister_type: String,
    pub package: Option<String>,
    pub main: Option<String>,
    pub dependencies: Option<Vec<String>>,
}

/// Network configuration in dfx.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct NetworkConfig {
    #[serde(rename = "type")]
    pub network_type: String,
    pub providers: Option<Vec<String>>,
    pub bind: Option<String>,
}

/// Canister IDs for different networks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CanisterIds {
    pub local: Option<String>,
    pub ic: Option<String>,
    pub testnet: Option<String>,
}

#[allow(dead_code)]
async fn load_dfx_config(project_root: &Path) -> Result<DfxConfig> {
    let dfx_path = project_root.join("dfx.json");

    let content = fs::read_to_string(&dfx_path)
        .await
        .with_context(|| format!("Failed to read dfx.json: {}", dfx_path.display()))?;

    let dfx_config: DfxConfig =
        serde_json::from_str(&content).with_context(|| "Failed to parse dfx.json")?;

    Ok(dfx_config)
}

#[allow(dead_code)]
async fn load_canister_ids(
    project_root: &Path,
) -> Result<std::collections::HashMap<String, CanisterIds>> {
    let canister_ids_path = project_root.join("canister_ids.json");

    if !canister_ids_path.exists() {
        return Ok(std::collections::HashMap::new());
    }

    let content = fs::read_to_string(&canister_ids_path)
        .await
        .with_context(|| {
            format!(
                "Failed to read canister_ids.json: {}",
                canister_ids_path.display()
            )
        })?;

    let canister_ids: std::collections::HashMap<String, CanisterIds> =
        serde_json::from_str(&content).with_context(|| "Failed to parse canister_ids.json")?;

    Ok(canister_ids)
}

/// Create a new project directory structure
#[allow(dead_code)]
pub async fn create_project_structure(project_path: &Path) -> Result<()> {
    // Create main directories
    let directories = vec!["src", "tests", ".dfx"];

    for dir in directories {
        let dir_path = project_path.join(dir);
        fs::create_dir_all(&dir_path)
            .await
            .with_context(|| format!("Failed to create directory: {}", dir_path.display()))?;
    }

    Ok(())
}

/// Validate project structure
#[allow(dead_code)]
pub fn validate_project_structure(project_root: &Path) -> Result<Vec<String>> {
    let mut issues = Vec::new();

    // Required files
    let required_files = vec!["Cargo.toml"];
    for file in required_files {
        if !project_root.join(file).exists() {
            issues.push(format!("Missing required file: {}", file));
        }
    }

    // Required directories
    let required_dirs = vec!["src"];
    for dir in required_dirs {
        if !project_root.join(dir).is_dir() {
            issues.push(format!("Missing required directory: {}", dir));
        }
    }

    // Check for lib.rs or main.rs
    let src_dir = project_root.join("src");
    if src_dir.exists() {
        if !src_dir.join("lib.rs").exists() && !src_dir.join("main.rs").exists() {
            issues.push("No lib.rs or main.rs found in src directory".to_string());
        }
    }

    Ok(issues)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn create_test_cargo_toml(dir: &Path, name: &str) -> Result<()> {
        let cargo_content = format!(
            r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"
description = "A test project"
authors = ["Test Author <test@example.com>"]
license = "MIT"

[lib]
crate-type = ["cdylib"]

[dependencies]
icarus = "0.9.0"
"#,
            name
        );

        fs::write(dir.join("Cargo.toml"), cargo_content).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_load_project_config() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        create_test_cargo_toml(project_path, "test-project")
            .await
            .unwrap();

        let config = load_project_config(project_path).await.unwrap();

        assert_eq!(config.name, "test-project");
        assert_eq!(config.version, "0.1.0");
        assert_eq!(config.description, Some("A test project".to_string()));
        assert_eq!(config.canister_type, "rust");
    }

    #[tokio::test]
    async fn test_is_icarus_project() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // Not an Icarus project initially
        assert!(!is_icarus_project(project_path).await);

        // Create Cargo.toml with icarus dependency
        create_test_cargo_toml(project_path, "test-project")
            .await
            .unwrap();

        // Now it should be detected as an Icarus project
        assert!(is_icarus_project(project_path).await);
    }

    #[tokio::test]
    async fn test_create_project_structure() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        create_project_structure(project_path).await.unwrap();

        assert!(project_path.join("src").is_dir());
        assert!(project_path.join("tests").is_dir());
        assert!(project_path.join(".dfx").is_dir());
    }

    #[test]
    fn test_validate_project_structure() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // Should have issues with empty directory
        let issues = validate_project_structure(project_path).unwrap();
        assert!(!issues.is_empty());
        assert!(issues
            .iter()
            .any(|issue| issue.contains("Missing required file: Cargo.toml")));
    }

    #[tokio::test]
    async fn test_project_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        create_test_cargo_toml(project_path, "test-project")
            .await
            .unwrap();

        let metadata = get_project_metadata(project_path).await.unwrap();

        assert_eq!(metadata.config.name, "test-project");
        assert!(metadata.dfx_config.is_none()); // No dfx.json created
        assert!(metadata.canister_ids.is_empty()); // No canister_ids.json created
    }
}
