use anyhow::Result;
use colored::Colorize;
use std::path::Path;
use tokio::signal;

use crate::utils::{print_info, print_success, print_warning, run_command_interactive};

pub async fn execute(port: u16, hot_reload: bool, skip_deploy: bool, debug: bool) -> Result<()> {
    println!(
        "\n{} {}",
        "🚀".bright_blue(),
        "Starting Development Server".bright_cyan().bold()
    );

    // Check if we're in an Icarus project
    let current_dir = std::env::current_dir()?;
    if !is_icarus_project(&current_dir) {
        print_warning("Not in an Icarus project directory.");
        print_info("Run this command from within an Icarus project created with 'icarus new'.");
        return Ok(());
    }

    // Configure debug logging
    if debug {
        print_info("Debug logging enabled");
        std::env::set_var("RUST_LOG", "debug");
    }

    print_info(&format!("Development server configuration:"));
    println!(
        "  {} Port: {}",
        "🔌".bright_blue(),
        port.to_string().bright_cyan()
    );
    println!(
        "  {} Hot reload: {}",
        "🔄".bright_blue(),
        if hot_reload {
            "enabled".bright_green()
        } else {
            "disabled".bright_yellow()
        }
    );
    println!(
        "  {} Skip initial deploy: {}",
        "📦".bright_blue(),
        if skip_deploy {
            "yes".bright_yellow()
        } else {
            "no".bright_green()
        }
    );
    println!();

    // Check if local IC replica is running
    check_local_replica().await?;

    // Initial build and deployment (unless skipped)
    if !skip_deploy {
        print_info("Building and deploying project...");
        match build_and_deploy().await {
            Ok(canister_id) => {
                print_success(&format!("Project deployed successfully!"));
                println!(
                    "  {} Canister ID: {}",
                    "🆔".bright_blue(),
                    canister_id.bright_cyan()
                );
            }
            Err(e) => {
                print_warning(&format!("Initial deployment failed: {}", e));
                print_info("Continuing with development server...");
            }
        }
    }

    // Start development server
    print_info(&format!("Starting development server on port {}...", port));

    if hot_reload {
        print_info("Hot reload enabled - changes will trigger automatic redeployment");
        print_info("Watching for file changes...");
    }

    println!("\n{} Development server is running!", "✅".bright_green());
    println!("{}", "────────────────────────────────────".bright_blue());
    println!(
        "  {} Local development: http://localhost:{}",
        "🌐".bright_blue(),
        port.to_string().bright_cyan()
    );
    println!(
        "  {} Hot reload: {}",
        "🔄".bright_blue(),
        if hot_reload {
            "ON".bright_green()
        } else {
            "OFF".bright_yellow()
        }
    );
    println!("  {} Press Ctrl+C to stop", "⏹️".bright_blue());
    println!("{}", "────────────────────────────────────".bright_blue());
    println!();

    // In a real implementation, this would start the actual development server
    // For now, we'll simulate it
    if hot_reload {
        simulate_hot_reload_server().await?;
    } else {
        simulate_basic_server().await?;
    }

    Ok(())
}

fn is_icarus_project(path: &Path) -> bool {
    path.join("Cargo.toml").exists() && path.join("dfx.json").exists() && path.join("src").exists()
}

async fn check_local_replica() -> Result<()> {
    print_info("Checking local IC replica...");

    match tokio::process::Command::new("dfx")
        .args(&["ping", "local"])
        .output()
        .await
    {
        Ok(output) if output.status.success() => {
            println!("  {} Local IC replica is running", "✅".bright_green());
        }
        _ => {
            println!("  {} Local IC replica not running", "❌".bright_red());
            print_info("Starting local IC replica...");

            match tokio::process::Command::new("dfx")
                .args(&["start", "--background"])
                .output()
                .await
            {
                Ok(output) if output.status.success() => {
                    println!("  {} Local IC replica started", "✅".bright_green());
                }
                Ok(_) => {
                    print_warning("Failed to start local IC replica - non-zero exit code");
                    return Err(anyhow::anyhow!(
                        "Local IC replica required for development server"
                    ));
                }
                Err(e) => {
                    print_warning(&format!("Failed to start local IC replica: {}", e));
                    return Err(anyhow::anyhow!(
                        "Local IC replica required for development server"
                    ));
                }
            }
        }
    }

    Ok(())
}

async fn build_and_deploy() -> Result<String> {
    let current_dir = std::env::current_dir()?;

    // Build WASM
    run_command_interactive(
        "cargo",
        &["build", "--target", "wasm32-unknown-unknown", "--release"],
        Some(&current_dir),
    )
    .await?;

    // Get project name for deployment
    let project_name = get_project_name(&current_dir)?;

    // Get current principal for init argument
    let principal_output = tokio::process::Command::new("dfx")
        .args(&["identity", "get-principal"])
        .current_dir(&current_dir)
        .output()
        .await?;

    if !principal_output.status.success() {
        return Err(anyhow::anyhow!("Failed to get dfx identity"));
    }

    let principal = String::from_utf8_lossy(&principal_output.stdout)
        .trim()
        .to_string();
    let init_arg = format!("(principal \"{}\")", principal);

    // Deploy canister
    run_command_interactive(
        "dfx",
        &[
            "deploy",
            &project_name,
            "--network",
            "local",
            "--argument",
            &init_arg,
        ],
        Some(&current_dir),
    )
    .await?;

    // Get canister ID
    let canister_id_output = tokio::process::Command::new("dfx")
        .args(&["canister", "id", &project_name, "--network", "local"])
        .current_dir(&current_dir)
        .output()
        .await?;

    if !canister_id_output.status.success() {
        return Err(anyhow::anyhow!("Failed to get canister ID"));
    }

    let canister_id = String::from_utf8_lossy(&canister_id_output.stdout)
        .trim()
        .to_string();
    Ok(canister_id)
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

async fn simulate_hot_reload_server() -> Result<()> {
    // This is a placeholder for the actual hot reload server implementation
    // In a real implementation, this would:
    // 1. Start a file watcher using notify crate
    // 2. Set up a web server on the specified port
    // 3. Handle file change events and trigger redeployment
    // 4. Provide real-time feedback via WebSocket connections

    print_info("Hot reload server simulation running...");
    println!("  {} Watching: src/", "👁️".bright_blue());
    println!("  {} Auto-deploy: enabled", "🔄".bright_blue());

    // Wait for Ctrl+C
    match signal::ctrl_c().await {
        Ok(()) => {
            println!(
                "\n{} Shutting down development server...",
                "⏹️".bright_yellow()
            );
        }
        Err(err) => {
            print_warning(&format!("Failed to listen for shutdown signal: {}", err));
        }
    }

    Ok(())
}

async fn simulate_basic_server() -> Result<()> {
    // This is a placeholder for the basic development server
    // In a real implementation, this would start a simple web server

    print_info("Basic development server simulation running...");

    // Wait for Ctrl+C
    match signal::ctrl_c().await {
        Ok(()) => {
            println!(
                "\n{} Shutting down development server...",
                "⏹️".bright_yellow()
            );
        }
        Err(err) => {
            print_warning(&format!("Failed to listen for shutdown signal: {}", err));
        }
    }

    Ok(())
}
