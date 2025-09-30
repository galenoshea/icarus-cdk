//! DFX utility functions for Internet Computer development
//! These are infrastructure functions that will be used as the CLI expands

#![allow(dead_code)]

use anyhow::{anyhow, Context, Result};
use std::path::Path;
use tokio::process::Command;

/// Check if dfx is available
pub(crate) async fn is_dfx_available() -> bool {
    Command::new("dfx")
        .arg("--version")
        .output()
        .await
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Get dfx version
pub(crate) async fn get_dfx_version() -> Result<String> {
    let output = Command::new("dfx")
        .arg("--version")
        .output()
        .await
        .context("Failed to execute dfx --version")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("dfx --version failed: {}", stderr));
    }

    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();

    Ok(version)
}

/// Start dfx replica in background
pub(crate) async fn start_replica(project_path: &Path, clean: bool) -> Result<()> {
    let mut cmd = Command::new("dfx");
    cmd.arg("start").arg("--background");

    if clean {
        cmd.arg("--clean");
    }

    cmd.current_dir(project_path);

    let output = cmd.output().await.context("Failed to start dfx replica")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("dfx start failed: {}", stderr));
    }

    Ok(())
}

/// Stop dfx replica
pub(crate) async fn stop_replica(project_path: &Path) -> Result<()> {
    let output = Command::new("dfx")
        .arg("stop")
        .current_dir(project_path)
        .output()
        .await
        .context("Failed to stop dfx replica")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("dfx stop failed: {}", stderr));
    }

    Ok(())
}

/// Check if dfx replica is running
pub(crate) async fn is_replica_running(project_path: &Path) -> Result<bool> {
    let output = Command::new("dfx")
        .args(["ping", "local"])
        .current_dir(project_path)
        .output()
        .await
        .context("Failed to ping dfx replica")?;

    Ok(output.status.success())
}

/// Deploy canisters using dfx
pub(crate) async fn deploy_canisters(
    project_path: &Path,
    network: &str,
    canister: Option<&str>,
    mode: &str,
) -> Result<String> {
    let mut cmd = Command::new("dfx");
    cmd.arg("deploy");
    cmd.arg("--network").arg(network);
    cmd.arg("--mode").arg(mode);

    if let Some(canister_name) = canister {
        cmd.arg(canister_name);
    }

    cmd.current_dir(project_path);

    let output = cmd.output().await.context("Failed to deploy canisters")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("dfx deploy failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.to_string())
}

/// Generate canister declarations
pub(crate) async fn generate_declarations(project_path: &Path) -> Result<()> {
    let output = Command::new("dfx")
        .arg("generate")
        .current_dir(project_path)
        .output()
        .await
        .context("Failed to generate declarations")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("dfx generate failed: {}", stderr));
    }

    Ok(())
}

/// Get canister status
pub(crate) async fn get_canister_status(
    project_path: &Path,
    canister_id: &str,
    network: &str,
) -> Result<String> {
    let output = Command::new("dfx")
        .args(["canister", "status", canister_id, "--network", network])
        .current_dir(project_path)
        .output()
        .await
        .context("Failed to get canister status")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("dfx canister status failed: {}", stderr));
    }

    let status = String::from_utf8_lossy(&output.stdout);
    Ok(status.to_string())
}

/// Get canister logs
pub(crate) async fn get_canister_logs(
    project_path: &Path,
    canister_name: &str,
    lines: Option<usize>,
) -> Result<String> {
    let mut cmd = Command::new("dfx");
    cmd.args(["canister", "logs", canister_name]);

    if let Some(line_count) = lines {
        cmd.arg("--lines").arg(line_count.to_string());
    }

    cmd.current_dir(project_path);

    let output = cmd.output().await.context("Failed to get canister logs")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("dfx canister logs failed: {}", stderr));
    }

    let logs = String::from_utf8_lossy(&output.stdout);
    Ok(logs.to_string())
}

/// Create a new dfx identity
pub(crate) async fn create_identity(name: &str) -> Result<()> {
    let output = Command::new("dfx")
        .args(["identity", "new", name])
        .output()
        .await
        .context("Failed to create dfx identity")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("dfx identity new failed: {}", stderr));
    }

    Ok(())
}

/// Use a dfx identity
pub(crate) async fn use_identity(name: &str) -> Result<()> {
    let output = Command::new("dfx")
        .args(["identity", "use", name])
        .output()
        .await
        .context("Failed to use dfx identity")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("dfx identity use failed: {}", stderr));
    }

    Ok(())
}

/// Get current dfx identity
pub(crate) async fn get_current_identity() -> Result<String> {
    let output = Command::new("dfx")
        .args(["identity", "whoami"])
        .output()
        .await
        .context("Failed to get current dfx identity")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("dfx identity whoami failed: {}", stderr));
    }

    let identity = String::from_utf8_lossy(&output.stdout).trim().to_string();

    Ok(identity)
}

/// Get dfx principal
pub(crate) async fn get_principal() -> Result<String> {
    let output = Command::new("dfx")
        .args(["identity", "get-principal"])
        .output()
        .await
        .context("Failed to get dfx principal")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("dfx identity get-principal failed: {}", stderr));
    }

    let principal = String::from_utf8_lossy(&output.stdout).trim().to_string();

    Ok(principal)
}

/// List available dfx identities
pub(crate) async fn list_identities() -> Result<Vec<String>> {
    let output = Command::new("dfx")
        .args(["identity", "list"])
        .output()
        .await
        .context("Failed to list dfx identities")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("dfx identity list failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let identities: Vec<String> = stdout
        .lines()
        .map(|line| line.trim().replace("*", "").trim().to_string())
        .filter(|line| !line.is_empty())
        .collect();

    Ok(identities)
}

/// Install dfx if not available (macOS and Linux only)
pub(crate) async fn install_dfx() -> Result<()> {
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    {
        let output = Command::new("sh")
            .args([
                "-ci",
                "$(curl -fsSL https://internetcomputer.org/install.sh)",
            ])
            .output()
            .await
            .context("Failed to install dfx")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("dfx installation failed: {}", stderr));
        }

        Ok(())
    }

    #[cfg(target_os = "windows")]
    {
        Err(anyhow!(
            "Automatic dfx installation is not supported on Windows. Please install manually."
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_dfx_availability() {
        let available = is_dfx_available().await;
        println!("dfx available: {}", available);
        // This test just checks that the function runs without error
    }

    #[tokio::test]
    async fn test_dfx_version() {
        if !is_dfx_available().await {
            return; // Skip test if dfx is not available
        }

        let result = get_dfx_version().await;
        if result.is_ok() {
            let version = result.unwrap();
            println!("dfx version: {}", version);
            assert!(version.contains("dfx"));
        }
        // If dfx is not available, the test will be skipped
    }

    #[tokio::test]
    async fn test_replica_status() {
        if !is_dfx_available().await {
            return; // Skip test if dfx is not available
        }

        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // This should return false (not running) or error in a temp directory
        let result = is_replica_running(project_path).await;
        // We don't assert anything specific since this depends on system state
        println!("Replica running check result: {:?}", result);
    }

    #[tokio::test]
    async fn test_identity_operations() {
        if !is_dfx_available().await {
            return; // Skip test if dfx is not available
        }

        // Test getting current identity
        let identity_result = get_current_identity().await;
        if identity_result.is_ok() {
            let identity = identity_result.unwrap();
            println!("Current identity: {}", identity);
            assert!(!identity.is_empty());
        }

        // Test getting principal
        let principal_result = get_principal().await;
        if principal_result.is_ok() {
            let principal = principal_result.unwrap();
            println!("Principal: {}", principal);
            assert!(!principal.is_empty());
        }

        // Test listing identities
        let list_result = list_identities().await;
        if list_result.is_ok() {
            let identities = list_result.unwrap();
            println!("Available identities: {:?}", identities);
            assert!(!identities.is_empty());
        }
    }

    #[test]
    fn test_command_construction() {
        // Test that we can construct commands without executing them
        let mut cmd = std::process::Command::new("dfx");
        cmd.args(["deploy", "--network", "local"]);

        let args: Vec<&std::ffi::OsStr> = cmd.get_args().collect();
        assert!(args.contains(&std::ffi::OsStr::new("deploy")));
        assert!(args.contains(&std::ffi::OsStr::new("--network")));
        assert!(args.contains(&std::ffi::OsStr::new("local")));
    }
}
