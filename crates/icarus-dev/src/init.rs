use anyhow::Result;
use colored::Colorize;
use std::path::Path;

use crate::utils::{print_info, print_success, print_warning, run_command};

pub async fn execute(skip_checks: bool, force: bool) -> Result<()> {
    println!("\n{} {}", "🔧".bright_blue(), "Initializing Development Environment".bright_cyan().bold());
    println!("{}", "Setting up your local development environment for Icarus MCP server development.\n".bright_white());

    // Check if we're in an Icarus project
    let current_dir = std::env::current_dir()?;
    let is_icarus_project = is_icarus_project(&current_dir);

    if !is_icarus_project {
        print_warning("Not in an Icarus project directory.");
        print_info("Initialize development environment from within an Icarus project created with 'icarus new'.");
        return Ok(());
    }

    print_info("Checking development environment...");

    // Check dependencies if not skipped
    if !skip_checks {
        check_development_dependencies().await?;
    }

    // Check if local IC replica is running
    check_local_replica().await?;

    // Initialize development configuration
    initialize_dev_config(force).await?;

    // Build project
    print_info("Building project...");
    let build_result = run_command(
        "cargo",
        &["build", "--target", "wasm32-unknown-unknown", "--release"],
        Some(&current_dir),
    )
    .await;

    match build_result {
        Ok(_) => print_success("Project built successfully!"),
        Err(e) => {
            print_warning(&format!("Build failed: {}", e));
            print_info("Fix build issues and run 'icarus dev init' again.");
        }
    }

    print_success("Development environment initialized!");
    println!("\n{} Next steps:", "💡".bright_yellow());
    println!("  {} Start development server: {}", "🚀".bright_green(), "icarus dev start".bright_cyan());
    println!("  {} Watch for changes: {}", "👀".bright_green(), "icarus dev watch".bright_cyan());
    println!("  {} Check status: {}", "📊".bright_green(), "icarus dev status".bright_cyan());

    Ok(())
}

fn is_icarus_project(path: &Path) -> bool {
    path.join("Cargo.toml").exists()
        && path.join("dfx.json").exists()
        && path.join("src").exists()
}

async fn check_development_dependencies() -> Result<()> {
    print_info("Checking development dependencies...");

    // Check dfx
    match run_command("dfx", &["--version"], None).await {
        Ok(_) => println!("  {} dfx installed", "✅".bright_green()),
        Err(_) => {
            println!("  {} dfx not found", "❌".bright_red());
            println!("    Install dfx: https://internetcomputer.org/docs/current/developer-docs/setup/install");
        }
    }

    // Check cargo
    match run_command("cargo", &["--version"], None).await {
        Ok(_) => println!("  {} cargo installed", "✅".bright_green()),
        Err(_) => {
            println!("  {} cargo not found", "❌".bright_red());
            println!("    Install Rust: https://rustup.rs/");
        }
    }

    // Check wasm32 target
    match run_command("rustup", &["target", "list", "--installed"], None).await {
        Ok(output) => {
            if output.contains("wasm32-unknown-unknown") {
                println!("  {} wasm32-unknown-unknown target installed", "✅".bright_green());
            } else {
                println!("  {} wasm32-unknown-unknown target missing", "❌".bright_red());
                print_info("Installing wasm32-unknown-unknown target...");
                run_command("rustup", &["target", "add", "wasm32-unknown-unknown"], None).await?;
                println!("  {} wasm32-unknown-unknown target installed", "✅".bright_green());
            }
        }
        Err(_) => {
            println!("  {} Cannot check rustup targets", "⚠️".bright_yellow());
        }
    }

    // Check ic-wasm (optional)
    match run_command("ic-wasm", &["--version"], None).await {
        Ok(_) => println!("  {} ic-wasm installed", "✅".bright_green()),
        Err(_) => {
            println!("  {} ic-wasm not found (optional)", "⚠️".bright_yellow());
            println!("    Install for WASM optimization: cargo install ic-wasm");
        }
    }

    // Check candid-extractor (optional)
    match run_command("candid-extractor", &["--version"], None).await {
        Ok(_) => println!("  {} candid-extractor installed", "✅".bright_green()),
        Err(_) => {
            println!("  {} candid-extractor not found (optional)", "⚠️".bright_yellow());
            println!("    Install for Candid generation: cargo install candid-extractor");
        }
    }

    Ok(())
}

async fn check_local_replica() -> Result<()> {
    print_info("Checking local IC replica...");

    match run_command("dfx", &["ping", "local"], None).await {
        Ok(_) => {
            println!("  {} Local IC replica is running", "✅".bright_green());
        }
        Err(_) => {
            println!("  {} Local IC replica not running", "⚠️".bright_yellow());
            print_info("Starting local IC replica...");

            match run_command("dfx", &["start", "--background"], None).await {
                Ok(_) => {
                    println!("  {} Local IC replica started", "✅".bright_green());
                }
                Err(e) => {
                    print_warning(&format!("Failed to start local IC replica: {}", e));
                    println!("    Try manually: dfx start --background");
                }
            }
        }
    }

    Ok(())
}

async fn initialize_dev_config(force: bool) -> Result<()> {
    let dev_config_path = Path::new(".icarus-dev.toml");

    if dev_config_path.exists() && !force {
        println!("  {} Development configuration already exists", "✅".bright_green());
        return Ok(());
    }

    print_info("Creating development configuration...");

    let config_content = r#"# Icarus Development Configuration

[dev]
# Development server settings
port = 8080
hot_reload = true
auto_deploy = true

# File watching settings
[watch]
patterns = ["src/**/*.rs", "Cargo.toml", "dfx.json"]
ignore_patterns = ["target/**", ".git/**"]
debounce_ms = 500

# Build settings
[build]
optimize_wasm = true
extract_candid = true
skip_tests = false

# Deployment settings
[deploy]
network = "local"
upgrade_on_change = true
preserve_state = true
"#;

    std::fs::write(dev_config_path, config_content)?;
    println!("  {} Created .icarus-dev.toml", "✅".bright_green());

    Ok(())
}