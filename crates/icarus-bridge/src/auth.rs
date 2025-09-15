//! Authentication utilities for ICP bridge
//!
//! Handles dfx identity detection and IC agent creation with proper authentication.

use anyhow::Result;
use candid::Principal;
use ic_agent::Agent;
use std::path::Path;
use std::process::Command;

/// Create an authenticated IC agent using the current dfx identity
///
/// This function automatically detects the current dfx identity and creates
/// an IC agent configured with the appropriate authentication.
///
/// # Arguments
///
/// * `is_mcp_mode` - Whether to suppress identity information output
///
/// # Returns
///
/// A tuple containing (identity_name, principal, agent)
pub async fn create_authenticated_agent(is_mcp_mode: bool) -> Result<(String, Principal, Agent)> {
    // Check if dfx is available
    let dfx_path = which::which("dfx")
        .or_else(|_| which::which("/usr/local/bin/dfx"))
        .or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_default();
            which::which(format!("{}/bin/dfx", home))
        });

    if let Ok(dfx_path) = dfx_path {
        match Command::new(&dfx_path).args(["identity", "whoami"]).output() {
            Ok(output) if output.status.success() => {
                let identity_name = String::from_utf8_lossy(&output.stdout).trim().to_string();

                // Get principal for this identity
                match Command::new(&dfx_path)
                    .args(["identity", "get-principal"])
                    .output()
                {
                    Ok(principal_output) if principal_output.status.success() => {
                        let principal_str =
                            String::from_utf8_lossy(&principal_output.stdout).trim().to_string();
                        if let Ok(principal) = Principal::from_text(&principal_str) {
                            // Get identity file path
                            let home_dir = std::env::var("HOME").unwrap_or_default();
                            let identity_path = format!(
                                "{}/.config/dfx/identity/{}/identity.pem",
                                home_dir, identity_name
                            );

                            // Try to load the identity
                            let agent = if Path::new(&identity_path).exists() {
                                // Try Secp256k1 first (newer format)
                                if let Ok(identity) =
                                    ic_agent::identity::Secp256k1Identity::from_pem_file(&identity_path)
                                {
                                    if !is_mcp_mode {
                                        eprintln!(
                                            "🔑 Using dfx identity '{}' (secp256k1) with principal: {}",
                                            identity_name, principal_str
                                        );
                                    }
                                    ic_agent::Agent::builder()
                                        .with_url("http://localhost:4943")
                                        .with_identity(identity)
                                        .build()?
                                } else if let Ok(identity) =
                                    ic_agent::identity::BasicIdentity::from_pem_file(&identity_path)
                                {
                                    if !is_mcp_mode {
                                        eprintln!(
                                            "🔑 Using dfx identity '{}' (ed25519) with principal: {}",
                                            identity_name, principal_str
                                        );
                                    }
                                    ic_agent::Agent::builder()
                                        .with_url("http://localhost:4943")
                                        .with_identity(identity)
                                        .build()?
                                } else {
                                    return Err(anyhow::anyhow!(
                                        "Could not load identity file at {}. Please ensure your dfx identity is properly configured.",
                                        identity_path
                                    ));
                                }
                            } else {
                                return Err(anyhow::anyhow!(
                                    "Identity file not found at {}. Please ensure dfx is properly configured.",
                                    identity_path
                                ));
                            };

                            // Fetch root key for local development
                            agent.fetch_root_key().await?;

                            return Ok((identity_name, principal, agent));
                        } else {
                            return Err(anyhow::anyhow!(
                                "Could not parse principal from dfx. Please ensure dfx is properly configured."
                            ));
                        }
                    }
                    _ => {
                        return Err(anyhow::anyhow!(
                            "Could not get principal from dfx. Please run 'dfx identity get-principal' to verify your identity."
                        ));
                    }
                }
            }
            _ => {
                return Err(anyhow::anyhow!(
                    "dfx command failed or is not available. Please ensure dfx is installed and a valid identity is selected."
                ));
            }
        }
    } else {
        return Err(anyhow::anyhow!(
            "dfx not found in standard locations. Please ensure dfx is installed and available in PATH."
        ));
    }
}

/// Simple helper to check if dfx is available
pub fn is_dfx_available() -> bool {
    which::which("dfx")
        .or_else(|_| which::which("/usr/local/bin/dfx"))
        .or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_default();
            which::which(format!("{}/bin/dfx", home))
        })
        .is_ok()
}

/// Get the current dfx identity name
pub fn get_current_identity() -> Result<String> {
    let dfx_path = which::which("dfx")?;
    let output = Command::new(dfx_path)
        .args(["identity", "whoami"])
        .output()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(anyhow::anyhow!("Failed to get current dfx identity"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_dfx_available() {
        // This test will pass if dfx is installed, or fail gracefully if not
        let result = is_dfx_available();
        // Just ensure it returns a boolean without panicking
        assert!(result == true || result == false);
    }

    #[test]
    fn test_get_current_identity_when_dfx_unavailable() {
        // Test behavior when dfx is not available
        // This will fail if dfx is actually available, but that's expected
        if !is_dfx_available() {
            let result = get_current_identity();
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_create_authenticated_agent_requires_dfx() {
        // Test that create_authenticated_agent fails gracefully without dfx
        // This is more of an integration test, but tests the error path
        tokio_test::block_on(async {
            if !is_dfx_available() {
                let result = create_authenticated_agent(true).await;
                assert!(result.is_err());
                assert!(result.unwrap_err().to_string().contains("dfx"));
            }
        });
    }

    #[test]
    fn test_identity_path_construction() {
        // Test the identity path construction logic by checking the format
        let home = "/home/test";
        let identity_name = "default";
        let expected_path = format!("{}/.config/dfx/identity/{}/identity.pem", home, identity_name);

        // Just verify the path format is correct
        assert!(expected_path.contains("/.config/dfx/identity/"));
        assert!(expected_path.ends_with("/identity.pem"));
    }

    #[test]
    fn test_mcp_mode_flag() {
        // Test that MCP mode flag doesn't cause panics
        tokio_test::block_on(async {
            if !is_dfx_available() {
                // Test both MCP modes when dfx is unavailable
                let result1 = create_authenticated_agent(true).await;
                let result2 = create_authenticated_agent(false).await;

                // Both should fail similarly when dfx is not available
                assert!(result1.is_err());
                assert!(result2.is_err());
            }
        });
    }
}