use anyhow::Result;
use serial_test::serial;
use std::fs;
use std::path::Path;

#[path = "../common/mod.rs"]
mod common;
use common::{CliRunner, SharedTestProject, TestProject};

#[tokio::test]
#[serial]
async fn test_deploy_fresh_canister() -> Result<()> {
    // This test verifies that deploying a new canister returns valid IDs in the URL
    let project = TestProject::new("test_deploy_fresh")?;

    // Build the project first
    let build_output = CliRunner::new()
        .current_dir(&project.path)
        .args(&["build"])
        .run()?;

    assert!(build_output.status.success(), "Build should succeed");

    // Deploy to local network
    let deploy_output = CliRunner::new()
        .current_dir(&project.path)
        .args(&["deploy", "--network", "local"])
        .run()?;

    assert!(deploy_output.status.success(), "Deploy should succeed");

    let output_str = String::from_utf8_lossy(&deploy_output.stdout);

    // Check that deployment succeeded
    assert!(
        output_str.contains("Deployed successfully")
            || output_str.contains("upgraded successfully"),
        "Should show deployment success"
    );

    // Check that URLs are displayed
    assert!(output_str.contains("URLs:"), "Should display URLs section");
    assert!(
        output_str.contains("Backend canister via Candid interface"),
        "Should show Candid UI URL"
    );

    // Extract the canister ID from the URL
    if let Some(url_line) = output_str
        .lines()
        .find(|line| line.contains("http://127.0.0.1:4943"))
    {
        // URL format: http://127.0.0.1:4943/?canisterId=xxx&id=yyy
        if let Some(id_part) = url_line.split("&id=").nth(1) {
            let canister_id = id_part.split_whitespace().next().unwrap_or("");

            // Verify the canister ID format (should be xxx-xxx-xxx-xxx-xxx format)
            assert!(
                canister_id.contains("-") && canister_id.len() > 10,
                "Canister ID should be valid format: {}",
                canister_id
            );

            // Verify canister actually exists by checking with dfx
            let status_output = std::process::Command::new("dfx")
                .args(&["canister", "status", &canister_id])
                .current_dir(&project.path)
                .output()?;

            assert!(
                status_output.status.success(),
                "Canister {} should exist and be queryable",
                canister_id
            );
        }
    }

    Ok(())
}

#[tokio::test]
#[serial]
async fn test_deploy_upgrade_existing() -> Result<()> {
    // This test verifies that upgrading an existing canister maintains correct IDs
    let project = TestProject::new("test_deploy_upgrade")?;

    // Build the project
    CliRunner::new()
        .current_dir(&project.path)
        .args(&["build"])
        .run()?;

    // First deployment
    let first_deploy = CliRunner::new()
        .current_dir(&project.path)
        .args(&["deploy", "--network", "local"])
        .run()?;

    assert!(first_deploy.status.success(), "First deploy should succeed");

    // Extract first canister ID
    let first_output = String::from_utf8_lossy(&first_deploy.stdout);
    let first_canister_id = extract_canister_id(&first_output);

    // Make a small change to trigger rebuild
    let lib_path = project.path.join("src").join("lib.rs");
    let content = fs::read_to_string(&lib_path)?;
    fs::write(&lib_path, format!("{}\n// Updated", content))?;

    // Rebuild
    CliRunner::new()
        .current_dir(&project.path)
        .args(&["build"])
        .run()?;

    // Second deployment (should upgrade)
    let second_deploy = CliRunner::new()
        .current_dir(&project.path)
        .args(&["deploy", "--network", "local"])
        .run()?;

    assert!(
        second_deploy.status.success(),
        "Second deploy should succeed"
    );

    let second_output = String::from_utf8_lossy(&second_deploy.stdout);
    assert!(
        second_output.contains("upgraded successfully"),
        "Should show upgrade message"
    );

    // Extract second canister ID
    let second_canister_id = extract_canister_id(&second_output);

    // Canister ID should remain the same after upgrade
    assert_eq!(
        first_canister_id, second_canister_id,
        "Canister ID should remain consistent after upgrade"
    );

    // Verify the ID is valid
    assert!(
        !second_canister_id.is_empty() && second_canister_id.contains("-"),
        "Canister ID should be valid: {}",
        second_canister_id
    );

    Ok(())
}

#[tokio::test]
#[serial]
async fn test_deploy_force_new() -> Result<()> {
    // This test verifies that --force creates a new canister with new ID
    let project = TestProject::new("test_deploy_force")?;

    // Build and deploy first time
    CliRunner::new()
        .current_dir(&project.path)
        .args(&["build"])
        .run()?;

    let first_deploy = CliRunner::new()
        .current_dir(&project.path)
        .args(&["deploy", "--network", "local"])
        .run()?;

    let first_output = String::from_utf8_lossy(&first_deploy.stdout);
    let first_id = extract_canister_id(&first_output);

    // Deploy with --force
    let force_deploy = CliRunner::new()
        .current_dir(&project.path)
        .args(&["deploy", "--network", "local", "--force"])
        .run()?;

    assert!(force_deploy.status.success(), "Force deploy should succeed");

    let force_output = String::from_utf8_lossy(&force_deploy.stdout);
    assert!(
        force_output.contains("Force deployed successfully"),
        "Should show force deployment message"
    );

    let force_id = extract_canister_id(&force_output);

    // Force deploy should create a new canister with different ID
    assert_ne!(
        first_id, force_id,
        "Force deploy should create new canister with different ID"
    );

    Ok(())
}

// Helper function to extract canister ID from deploy output
fn extract_canister_id(output: &str) -> String {
    output
        .lines()
        .find(|line| line.contains("http://127.0.0.1:4943"))
        .and_then(|line| line.split("&id=").nth(1))
        .and_then(|id_part| id_part.split_whitespace().next())
        .unwrap_or("")
        .to_string()
}

#[tokio::test]
#[serial]
async fn test_deploy_url_validity() -> Result<()> {
    // This test specifically checks that both IDs in the URL are valid
    let project = TestProject::new("test_deploy_url")?;

    // Build and deploy
    CliRunner::new()
        .current_dir(&project.path)
        .args(&["build"])
        .run()?;

    let deploy_output = CliRunner::new()
        .current_dir(&project.path)
        .args(&["deploy", "--network", "local"])
        .run()?;

    let output = String::from_utf8_lossy(&deploy_output.stdout);

    // Extract the full URL
    if let Some(url_line) = output
        .lines()
        .find(|line| line.contains("http://127.0.0.1:4943"))
    {
        // Parse both canister IDs from URL
        let url_parts: Vec<&str> = url_line.split("?canisterId=").collect();
        if url_parts.len() > 1 {
            let params = url_parts[1];

            // Extract Candid UI ID
            let candid_ui_id = params.split("&id=").next().unwrap_or("");
            assert!(
                !candid_ui_id.is_empty() && candid_ui_id.contains("-"),
                "Candid UI ID should be valid: {}",
                candid_ui_id
            );

            // Extract project canister ID
            if let Some(id_part) = params.split("&id=").nth(1) {
                let project_id = id_part.split_whitespace().next().unwrap_or("");
                assert!(
                    !project_id.is_empty() && project_id.contains("-"),
                    "Project canister ID should be valid: {}",
                    project_id
                );

                // Verify both canisters exist
                for (name, id) in [("__Candid_UI", candid_ui_id), ("project", project_id)] {
                    let status = std::process::Command::new("dfx")
                        .args(&["canister", "status", id])
                        .output();

                    if let Ok(output) = status {
                        assert!(
                            output.status.success()
                                || String::from_utf8_lossy(&output.stderr).contains("Running"),
                            "{} canister {} should exist",
                            name,
                            id
                        );
                    }
                }
            }
        }
    } else {
        panic!("No URL found in deploy output");
    }

    Ok(())
}
