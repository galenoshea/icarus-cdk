use anyhow::Result;
use colored::Colorize;
use std::path::Path;

use crate::utils::{print_info, print_success, print_warning};

pub async fn execute(clean: bool, yes: bool) -> Result<()> {
    println!("\n{} {}", "🔄".bright_blue(), "Reset Development Environment".bright_cyan().bold());
    println!("{}", "This will reset your local development environment.\n".bright_white());

    // Check if we're in an Icarus project
    let current_dir = std::env::current_dir()?;
    if !is_icarus_project(&current_dir) {
        print_warning("Not in an Icarus project directory.");
        print_info("Run this command from within an Icarus project created with 'icarus new'.");
        return Ok(());
    }

    // Show what will be reset
    print_info("Reset operations:");

    if clean {
        println!("  {} Remove all local canisters", "🗑️".bright_red());
        println!("  {} Clear canister state", "💾".bright_red());
        println!("  {} Stop local IC replica", "⏹️".bright_red());
    }

    println!("  {} Clear build artifacts", "🧹".bright_yellow());
    println!("  {} Reset development configuration", "⚙️".bright_yellow());
    println!("  {} Clear temporary files", "📁".bright_yellow());

    // Confirmation unless --yes flag is provided
    if !yes {
        println!("\n{} This operation cannot be undone.", "⚠️".bright_yellow());

        use std::io::{self, Write};
        print!("Continue with reset? (y/N): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().to_lowercase().starts_with('y') {
            print_info("Reset cancelled.");
            return Ok(());
        }
    }

    println!();
    print_info("Starting reset process...");

    // Stop local IC replica if requested
    if clean {
        print_info("Stopping local IC replica...");
        match tokio::process::Command::new("dfx")
            .args(&["stop"])
            .output()
            .await
        {
            Ok(output) if output.status.success() => {
                println!("  {} Local IC replica stopped", "✅".bright_green());
            }
            Ok(_) => {
                println!("  {} Local IC replica was not running", "ℹ️".bright_blue());
            }
            Err(e) => {
                print_warning(&format!("Could not stop local IC replica: {}", e));
            }
        }
    }

    // Clear build artifacts
    print_info("Clearing build artifacts...");
    clear_build_artifacts(&current_dir).await?;

    // Reset development configuration
    print_info("Resetting development configuration...");
    reset_dev_configuration(&current_dir).await?;

    // Clear temporary files
    print_info("Clearing temporary files...");
    clear_temporary_files(&current_dir).await?;

    if clean {
        // Clean dfx state
        print_info("Cleaning dfx state...");
        clean_dfx_state(&current_dir).await?;

        // Remove canister_ids.json
        let canister_ids_path = current_dir.join("canister_ids.json");
        if canister_ids_path.exists() {
            if let Err(e) = std::fs::remove_file(&canister_ids_path) {
                print_warning(&format!("Could not remove canister_ids.json: {}", e));
            } else {
                println!("  {} Removed canister_ids.json", "✅".bright_green());
            }
        }
    }

    print_success("Development environment reset complete!");

    println!("\n{} Next steps:", "💡".bright_yellow());
    println!("  {} Reinitialize: {}", "🔧".bright_green(), "icarus dev init".bright_cyan());

    if clean {
        println!("  {} Start replica: {}", "🚀".bright_green(), "dfx start --background".bright_cyan());
    }

    println!("  {} Deploy project: {}", "📦".bright_green(), "icarus deploy --network local".bright_cyan());

    Ok(())
}

fn is_icarus_project(path: &Path) -> bool {
    path.join("Cargo.toml").exists()
        && path.join("dfx.json").exists()
        && path.join("src").exists()
}

async fn clear_build_artifacts(project_dir: &Path) -> Result<()> {
    let target_dir = project_dir.join("target");

    if target_dir.exists() {
        match tokio::process::Command::new("cargo")
            .args(&["clean"])
            .current_dir(project_dir)
            .output()
            .await
        {
            Ok(output) if output.status.success() => {
                println!("  {} Cleared cargo build artifacts", "✅".bright_green());
            }
            _ => {
                // Try to remove target directory manually
                if let Err(e) = std::fs::remove_dir_all(&target_dir) {
                    print_warning(&format!("Could not remove target directory: {}", e));
                } else {
                    println!("  {} Removed target directory", "✅".bright_green());
                }
            }
        }
    } else {
        println!("  {} No build artifacts to clear", "ℹ️".bright_blue());
    }

    Ok(())
}

async fn reset_dev_configuration(project_dir: &Path) -> Result<()> {
    let dev_config_path = project_dir.join(".icarus-dev.toml");

    if dev_config_path.exists() {
        if let Err(e) = std::fs::remove_file(&dev_config_path) {
            print_warning(&format!("Could not remove .icarus-dev.toml: {}", e));
        } else {
            println!("  {} Removed .icarus-dev.toml", "✅".bright_green());
        }
    } else {
        println!("  {} No development configuration to reset", "ℹ️".bright_blue());
    }

    Ok(())
}

async fn clear_temporary_files(project_dir: &Path) -> Result<()> {
    let temp_extensions = [".tmp", ".log"];
    let temp_files = [".DS_Store", "Thumbs.db"];

    let mut cleared_count = 0;

    // Check for temporary files by extension
    if let Ok(entries) = std::fs::read_dir(project_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let file_name = path.file_name().unwrap_or_default().to_string_lossy();

                // Check extensions
                let should_remove = temp_extensions.iter().any(|ext| file_name.ends_with(ext))
                    || temp_files.iter().any(|name| file_name == *name);

                if should_remove {
                    if let Err(e) = std::fs::remove_file(&path) {
                        print_warning(&format!("Could not remove {:?}: {}", path, e));
                    } else {
                        cleared_count += 1;
                    }
                }
            }
        }
    }

    if cleared_count > 0 {
        println!("  {} Removed {} temporary files", "✅".bright_green(), cleared_count);
    } else {
        println!("  {} No temporary files to clear", "ℹ️".bright_blue());
    }

    Ok(())
}

async fn clean_dfx_state(project_dir: &Path) -> Result<()> {
    let dfx_dir = project_dir.join(".dfx");

    if dfx_dir.exists() {
        if let Err(e) = std::fs::remove_dir_all(&dfx_dir) {
            print_warning(&format!("Could not remove .dfx directory: {}", e));
        } else {
            println!("  {} Removed .dfx directory", "✅".bright_green());
        }
    } else {
        println!("  {} No dfx state to clean", "ℹ️".bright_blue());
    }

    Ok(())
}