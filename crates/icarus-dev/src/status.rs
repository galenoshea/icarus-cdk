use anyhow::Result;
use colored::Colorize;
use std::path::Path;

use crate::utils::print_warning;

pub async fn execute(detailed: bool) -> Result<()> {
    println!(
        "\n{} {}",
        "📊".bright_blue(),
        "Development Environment Status".bright_cyan().bold()
    );
    println!(
        "{}",
        "Current status of your Icarus development environment.\n".bright_white()
    );

    // Check if we're in an Icarus project
    let current_dir = std::env::current_dir()?;
    let is_icarus_project = is_icarus_project(&current_dir);

    println!(
        "{} {}",
        "📁".bright_blue(),
        "Project Status".bright_cyan().bold()
    );
    println!("{}", "──────────────────".bright_blue());

    if is_icarus_project {
        println!(
            "  {} Status: {}",
            "📦".bright_green(),
            "Icarus Project".bright_green()
        );

        // Get project name
        match get_project_name(&current_dir) {
            Ok(name) => println!("  {} Name: {}", "🏷️".bright_blue(), name.bright_cyan()),
            Err(_) => println!(
                "  {} Name: {}",
                "🏷️".bright_yellow(),
                "Unknown".bright_yellow()
            ),
        }

        // Check if built
        let wasm_exists = check_wasm_exists(&current_dir).await;
        let build_status = if wasm_exists {
            "Built".bright_green()
        } else {
            "Not Built".bright_yellow()
        };
        println!("  {} Build: {}", "🔨".bright_blue(), build_status);
    } else {
        println!(
            "  {} Status: {}",
            "📁".bright_yellow(),
            "Not an Icarus Project".bright_yellow()
        );
        print_warning("Run this command from within an Icarus project directory.");
    }

    println!();

    // Check development tools
    println!(
        "{} {}",
        "🔧".bright_blue(),
        "Development Tools".bright_cyan().bold()
    );
    println!("{}", "──────────────────".bright_blue());
    check_dev_tools().await;

    println!();

    // Check local IC status
    println!(
        "{} {}",
        "🌐".bright_blue(),
        "Local Internet Computer".bright_cyan().bold()
    );
    println!("{}", "──────────────────".bright_blue());
    check_local_ic_status().await?;

    // If detailed and in project, show more info
    if detailed && is_icarus_project {
        println!();
        show_detailed_project_info(&current_dir).await?;
    }

    // Check development configuration
    println!();
    check_dev_configuration(&current_dir);

    Ok(())
}

fn is_icarus_project(path: &Path) -> bool {
    path.join("Cargo.toml").exists() && path.join("dfx.json").exists() && path.join("src").exists()
}

fn get_project_name(project_dir: &Path) -> Result<String> {
    let cargo_toml = project_dir.join("Cargo.toml");
    let content = std::fs::read_to_string(&cargo_toml)?;
    let toml: toml::Value = toml::from_str(&content)?;

    toml.get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow::anyhow!("Could not find package name in Cargo.toml"))
}

async fn check_wasm_exists(project_dir: &Path) -> bool {
    if let Ok(project_name) = get_project_name(project_dir) {
        let wasm_name = project_name.replace('-', "_");
        let wasm_path = project_dir
            .join("target")
            .join("wasm32-unknown-unknown")
            .join("release")
            .join(format!("{}.wasm", wasm_name));
        wasm_path.exists()
    } else {
        false
    }
}

async fn check_dev_tools() {
    let tools = [
        ("dfx", "dfx --version"),
        ("cargo", "cargo --version"),
        ("rustc", "rustc --version"),
    ];

    for (name, command) in &tools {
        let parts: Vec<&str> = command.split_whitespace().collect();
        let cmd = parts[0];
        let args = &parts[1..];

        match tokio::process::Command::new(cmd).args(args).output().await {
            Ok(output) if output.status.success() => {
                let version = String::from_utf8_lossy(&output.stdout);
                let version_line = version.lines().next().unwrap_or("unknown").trim();
                let version_str = version_line.to_string();
                println!(
                    "  {} {}: {}",
                    "✅".bright_green(),
                    name,
                    version_str.bright_cyan()
                );
            }
            _ => {
                println!(
                    "  {} {}: {}",
                    "❌".bright_red(),
                    name,
                    "Not found".bright_red()
                );
            }
        }
    }

    // Check optional tools
    let optional_tools = [
        ("ic-wasm", "ic-wasm --version", "WASM optimization"),
        (
            "candid-extractor",
            "candid-extractor --version",
            "Candid generation",
        ),
    ];

    for (name, command, description) in &optional_tools {
        let parts: Vec<&str> = command.split_whitespace().collect();
        let cmd = parts[0];
        let args = &parts[1..];

        match tokio::process::Command::new(cmd).args(args).output().await {
            Ok(output) if output.status.success() => {
                let version = String::from_utf8_lossy(&output.stdout);
                let version_line = version.lines().next().unwrap_or("unknown").trim();
                let version_str = version_line.to_string();
                println!(
                    "  {} {} ({}): {}",
                    "✅".bright_green(),
                    name,
                    description,
                    version_str.bright_cyan()
                );
            }
            _ => {
                println!(
                    "  {} {} ({}): {}",
                    "⚠️".bright_yellow(),
                    name,
                    description,
                    "Not installed".bright_yellow()
                );
            }
        }
    }
}

async fn check_local_ic_status() -> Result<()> {
    // Check if dfx is running
    match tokio::process::Command::new("dfx")
        .args(&["ping", "local"])
        .output()
        .await
    {
        Ok(output) if output.status.success() => {
            println!(
                "  {} Local replica: {}",
                "✅".bright_green(),
                "Running".bright_green()
            );

            // Get replica info if available
            if let Ok(info_output) = tokio::process::Command::new("dfx")
                .args(&["info"])
                .output()
                .await
            {
                if info_output.status.success() {
                    let info = String::from_utf8_lossy(&info_output.stdout);
                    for line in info.lines() {
                        if line.contains("replica") && line.contains("http://") {
                            let url = line
                                .split_whitespace()
                                .find(|s| s.starts_with("http://"))
                                .unwrap_or("http://localhost:4943");
                            println!("  {} URL: {}", "🔗".bright_blue(), url.bright_cyan());
                            break;
                        }
                    }
                }
            }
        }
        _ => {
            println!(
                "  {} Local replica: {}",
                "❌".bright_red(),
                "Not running".bright_red()
            );
            println!(
                "    💡 Start with: {}",
                "dfx start --background".bright_cyan()
            );
        }
    }

    // Check identity
    match tokio::process::Command::new("dfx")
        .args(&["identity", "whoami"])
        .output()
        .await
    {
        Ok(output) if output.status.success() => {
            let identity = String::from_utf8_lossy(&output.stdout);
            let identity_str = identity.trim().to_string();
            println!(
                "  {} Identity: {}",
                "👤".bright_blue(),
                identity_str.bright_cyan()
            );
        }
        _ => {
            println!(
                "  {} Identity: {}",
                "⚠️".bright_yellow(),
                "Unknown".bright_yellow()
            );
        }
    }

    Ok(())
}

async fn show_detailed_project_info(project_dir: &Path) -> Result<()> {
    println!(
        "{} {}",
        "📋".bright_blue(),
        "Detailed Project Information".bright_cyan().bold()
    );
    println!("{}", "──────────────────".bright_blue());

    // Check canister information if dfx is available
    if let Ok(project_name) = get_project_name(project_dir) {
        // Try to get canister info
        match tokio::process::Command::new("dfx")
            .args(&["canister", "id", &project_name, "--network", "local"])
            .current_dir(project_dir)
            .output()
            .await
        {
            Ok(output) if output.status.success() => {
                let canister_id = String::from_utf8_lossy(&output.stdout);
                let canister_id_str = canister_id.trim().to_string();
                println!(
                    "  {} Local Canister ID: {}",
                    "🆔".bright_blue(),
                    canister_id_str.bright_cyan()
                );

                // Try to get canister status
                match tokio::process::Command::new("dfx")
                    .args(&["canister", "status", &canister_id_str, "--network", "local"])
                    .current_dir(project_dir)
                    .output()
                    .await
                {
                    Ok(status_output) if status_output.status.success() => {
                        let status = String::from_utf8_lossy(&status_output.stdout);
                        for line in status.lines() {
                            if line.contains("Status:")
                                || line.contains("Memory:")
                                || line.contains("Balance:")
                            {
                                println!("  {} {}", "📊".bright_blue(), line.trim().bright_white());
                            }
                        }
                    }
                    _ => {
                        println!(
                            "  {} Canister status: {}",
                            "⚠️".bright_yellow(),
                            "Could not retrieve".bright_yellow()
                        );
                    }
                }
            }
            _ => {
                println!(
                    "  {} Local Canister: {}",
                    "📦".bright_yellow(),
                    "Not deployed".bright_yellow()
                );
            }
        }
    }

    // Check file sizes
    if let Ok(metadata) = std::fs::metadata(project_dir.join("Cargo.toml")) {
        println!(
            "  {} Cargo.toml: {} bytes",
            "📄".bright_blue(),
            metadata.len().to_string().bright_cyan()
        );
    }

    if let Ok(metadata) = std::fs::metadata(project_dir.join("dfx.json")) {
        println!(
            "  {} dfx.json: {} bytes",
            "📄".bright_blue(),
            metadata.len().to_string().bright_cyan()
        );
    }

    // Check WASM size if exists
    if let Ok(project_name) = get_project_name(project_dir) {
        let wasm_name = project_name.replace('-', "_");
        let wasm_path = project_dir
            .join("target")
            .join("wasm32-unknown-unknown")
            .join("release")
            .join(format!("{}.wasm", wasm_name));

        if let Ok(metadata) = std::fs::metadata(&wasm_path) {
            println!(
                "  {} WASM size: {} bytes",
                "⚙️".bright_blue(),
                metadata.len().to_string().bright_cyan()
            );
        }
    }

    Ok(())
}

fn check_dev_configuration(project_dir: &Path) {
    println!(
        "{} {}",
        "⚙️".bright_blue(),
        "Development Configuration".bright_cyan().bold()
    );
    println!("{}", "──────────────────".bright_blue());

    let dev_config_path = project_dir.join(".icarus-dev.toml");
    if dev_config_path.exists() {
        println!(
            "  {} Development config: {}",
            "✅".bright_green(),
            "Found".bright_green()
        );

        if let Ok(content) = std::fs::read_to_string(&dev_config_path) {
            println!(
                "  {} Config size: {} bytes",
                "📄".bright_blue(),
                content.len().to_string().bright_cyan()
            );
        }
    } else {
        println!(
            "  {} Development config: {}",
            "⚠️".bright_yellow(),
            "Not found".bright_yellow()
        );
        println!(
            "    💡 Initialize with: {}",
            "icarus dev init".bright_cyan()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_is_icarus_project_with_all_files() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // Create required files
        fs::write(
            project_path.join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();
        fs::write(project_path.join("dfx.json"), "{}").unwrap();
        fs::create_dir(project_path.join("src")).unwrap();

        assert!(is_icarus_project(project_path));
    }

    #[test]
    fn test_is_icarus_project_missing_cargo_toml() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // Create only some files
        fs::write(project_path.join("dfx.json"), "{}").unwrap();
        fs::create_dir(project_path.join("src")).unwrap();

        assert!(!is_icarus_project(project_path));
    }

    #[test]
    fn test_is_icarus_project_missing_dfx_json() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // Create only some files
        fs::write(
            project_path.join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();
        fs::create_dir(project_path.join("src")).unwrap();

        assert!(!is_icarus_project(project_path));
    }

    #[test]
    fn test_is_icarus_project_missing_src() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // Create only some files
        fs::write(
            project_path.join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();
        fs::write(project_path.join("dfx.json"), "{}").unwrap();

        assert!(!is_icarus_project(project_path));
    }

    #[test]
    fn test_get_project_name_success() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        let cargo_toml_content = r#"
[package]
name = "my-test-project"
version = "0.1.0"
"#;
        fs::write(project_path.join("Cargo.toml"), cargo_toml_content).unwrap();

        let result = get_project_name(project_path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "my-test-project");
    }

    #[test]
    fn test_get_project_name_missing_file() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // No Cargo.toml file
        let result = get_project_name(project_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_project_name_invalid_toml() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // Invalid TOML content
        fs::write(project_path.join("Cargo.toml"), "invalid toml content [[[").unwrap();

        let result = get_project_name(project_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_project_name_missing_package_section() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        let cargo_toml_content = r#"
[dependencies]
serde = "1.0"
"#;
        fs::write(project_path.join("Cargo.toml"), cargo_toml_content).unwrap();

        let result = get_project_name(project_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_project_name_missing_name_field() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        let cargo_toml_content = r#"
[package]
version = "0.1.0"
"#;
        fs::write(project_path.join("Cargo.toml"), cargo_toml_content).unwrap();

        let result = get_project_name(project_path);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_check_wasm_exists_with_project() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // Create project structure
        let cargo_toml_content = r#"
[package]
name = "test-project"
version = "0.1.0"
"#;
        fs::write(project_path.join("Cargo.toml"), cargo_toml_content).unwrap();

        // Create target directory structure
        let wasm_dir = project_path
            .join("target")
            .join("wasm32-unknown-unknown")
            .join("release");
        fs::create_dir_all(&wasm_dir).unwrap();

        // Test without WASM file
        let result = check_wasm_exists(project_path).await;
        assert!(!result);

        // Create WASM file (project name with dashes becomes underscores)
        fs::write(wasm_dir.join("test_project.wasm"), "fake wasm content").unwrap();

        // Test with WASM file
        let result = check_wasm_exists(project_path).await;
        assert!(result);
    }

    #[tokio::test]
    async fn test_check_wasm_exists_no_project_name() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // Create invalid Cargo.toml
        fs::write(project_path.join("Cargo.toml"), "invalid content").unwrap();

        let result = check_wasm_exists(project_path).await;
        assert!(!result);
    }

    #[test]
    fn test_check_dev_configuration_exists() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // Create dev config file
        let config_content = r#"
[watch]
patterns = ["src/**/*.rs"]
delay = 1000
"#;
        fs::write(project_path.join(".icarus-dev.toml"), config_content).unwrap();

        // This function prints to stdout, so we can't easily test output
        // but we can test it doesn't panic
        check_dev_configuration(project_path);
    }

    #[test]
    fn test_check_dev_configuration_missing() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // No dev config file
        check_dev_configuration(project_path);
    }

    #[test]
    fn test_project_name_with_dashes() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        let cargo_toml_content = r#"
[package]
name = "my-complex-project-name"
version = "0.1.0"
"#;
        fs::write(project_path.join("Cargo.toml"), cargo_toml_content).unwrap();

        let result = get_project_name(project_path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "my-complex-project-name");
    }

    #[test]
    fn test_empty_project_directory() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // Empty directory
        assert!(!is_icarus_project(project_path));

        let result = get_project_name(project_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_project_with_extra_files() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // Create required files plus extras
        fs::write(
            project_path.join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();
        fs::write(project_path.join("dfx.json"), "{}").unwrap();
        fs::create_dir(project_path.join("src")).unwrap();
        fs::write(project_path.join("README.md"), "# Test project").unwrap();
        fs::create_dir(project_path.join("docs")).unwrap();

        // Should still be recognized as Icarus project
        assert!(is_icarus_project(project_path));
    }
}
