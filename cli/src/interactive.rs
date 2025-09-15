//! Interactive CLI wizard for guided operations
//!
//! Provides step-by-step guidance for common tasks like project setup,
//! deployment, and configuration management.

use anyhow::{Context, Result};
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};
use std::path::Path;

use crate::commands;
use crate::config::IcarusConfig;

/// Interactive wizard for common Icarus operations
pub struct InteractiveWizard {
    theme: ColorfulTheme,
}

impl InteractiveWizard {
    pub fn new() -> Self {
        Self {
            theme: ColorfulTheme::default(),
        }
    }

    /// Main interactive wizard entry point
    pub async fn run(&self) -> Result<()> {
        self.show_welcome_banner();

        // Show quick system status
        self.show_quick_status().await;

        loop {
            let actions = vec![
                "🆕 Create a new MCP server project",
                "📦 Deploy project to ICP",
                "⚙️  Configure project settings",
                "🔍 Analyze project structure",
                "🏥 Health check and diagnostics",
                "❌ Exit wizard",
            ];

            let selection = Select::with_theme(&self.theme)
                .with_prompt("What would you like to do?")
                .items(&actions)
                .default(0)
                .interact()?;

            match selection {
                0 => self.wizard_new_project().await?,
                1 => self.wizard_deploy().await?,
                2 => self.wizard_configure().await?,
                3 => self.wizard_analyze().await?,
                4 => self.wizard_health_check().await?,
                5 => {
                    println!("\n{} {}", "👋".bright_blue(), "Thanks for using Icarus! Happy coding!".bright_green());
                    break;
                }
                _ => unreachable!(),
            }

            println!("\n{}", "─".repeat(60).bright_black());
        }

        Ok(())
    }

    /// Wizard for creating a new project
    async fn wizard_new_project(&self) -> Result<()> {
        println!("\n{} {}", "🆕".bright_blue(), "New Project Wizard".bright_cyan().bold());
        println!("{}", "Let's create your new MCP server project!\n".bright_white());

        // Project name
        let name: String = Input::with_theme(&self.theme)
            .with_prompt("Project name")
            .validate_with(|input: &String| -> Result<(), &str> {
                if input.trim().is_empty() {
                    Err("Project name cannot be empty")
                } else if !input.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
                    Err("Project name can only contain letters, numbers, hyphens, and underscores")
                } else {
                    Ok(())
                }
            })
            .interact_text()?;

        // Project directory
        let current_dir = std::env::current_dir()
            .context("Failed to get current directory")?
            .display()
            .to_string();

        let default_path = format!("{}/{}", current_dir, name);
        let path: String = Input::with_theme(&self.theme)
            .with_prompt("Project directory")
            .default(default_path.clone())
            .interact_text()?;

        // Check if directory exists
        if Path::new(&path).exists() {
            let overwrite = Confirm::with_theme(&self.theme)
                .with_prompt(format!("Directory '{}' already exists. Continue anyway?", path))
                .default(false)
                .interact()?;

            if !overwrite {
                println!("{} Project creation cancelled.", "⚠️".bright_yellow());
                return Ok(());
            }
        }

        // Include tests
        let with_tests = Confirm::with_theme(&self.theme)
            .with_prompt("Include test files and dependencies?")
            .default(true)
            .interact()?;

        // Local SDK option
        let use_local_sdk = Confirm::with_theme(&self.theme)
            .with_prompt("Use local SDK for development? (for SDK contributors)")
            .default(false)
            .interact()?;

        let local_sdk = if use_local_sdk {
            Some(Input::<String>::with_theme(&self.theme)
                .with_prompt("Path to local SDK")
                .interact_text()?)
        } else {
            None
        };

        // Show summary
        println!("\n{} {}", "📋".bright_blue(), "Project Summary".bright_cyan().bold());
        println!("  {} {}", "Name:".bright_white(), name.bright_green());
        println!("  {} {}", "Path:".bright_white(), path.bright_green());
        println!("  {} {}", "Tests:".bright_white(), if with_tests { "Yes".bright_green() } else { "No".bright_red() });
        if let Some(ref sdk_path) = local_sdk {
            println!("  {} {}", "Local SDK:".bright_white(), sdk_path.bright_green());
        }

        let confirm = Confirm::with_theme(&self.theme)
            .with_prompt("\nCreate this project?")
            .default(true)
            .interact()?;

        if confirm {
            println!("\n{} Creating project...", "⏳".bright_blue());

            // Use existing new command function
            match commands::new::execute(name.clone(), Some(path.clone()), local_sdk, with_tests).await {
                Ok(_) => {
                    println!("{} {} {}", "✅".bright_green(), "Successfully created project".bright_green(), name.bright_cyan().bold());
                    println!("\n{} Next steps:", "💡".bright_yellow());
                    println!("  {} {}", "1.".bright_white(), format!("cd {}", path).bright_cyan());
                    println!("  {} {}", "2.".bright_white(), "icarus deploy --network local".bright_cyan());
                    println!("  {} {}", "3.".bright_white(), "icarus mcp start".bright_cyan());
                }
                Err(e) => {
                    println!("{} Failed to create project: {}", "❌".bright_red(), e);
                }
            }
        } else {
            println!("{} Project creation cancelled.", "⚠️".bright_yellow());
        }

        Ok(())
    }

    /// Wizard for deployment
    async fn wizard_deploy(&self) -> Result<()> {
        println!("\n{} {}", "📦".bright_blue(), "Deployment Wizard".bright_cyan().bold());
        println!("{}", "Let's deploy your MCP server to the Internet Computer!\n".bright_white());

        // Check if we're in a project directory
        if !Path::new("Cargo.toml").exists() {
            println!("{} {} {}",
                "⚠️".bright_yellow(),
                "No Cargo.toml found in current directory.".bright_yellow(),
                "Are you in an Icarus project?".bright_white()
            );
            return Ok(());
        }

        // Network selection
        let networks = vec!["local", "ic"];
        let network = Select::with_theme(&self.theme)
            .with_prompt("Select deployment network")
            .items(&networks)
            .default(0)
            .interact()?;

        let network_name = networks[network];

        // Deployment options
        let force = if network_name == "local" {
            Confirm::with_theme(&self.theme)
                .with_prompt("Force new deployment? (deletes existing canister)")
                .default(false)
                .interact()?
        } else {
            false
        };

        // Show deployment summary
        println!("\n{} {}", "📋".bright_blue(), "Deployment Summary".bright_cyan().bold());
        println!("  {} {}", "Network:".bright_white(), network_name.bright_green());
        println!("  {} {}", "Force:".bright_white(), if force { "Yes".bright_yellow() } else { "No".bright_green() });

        let confirm = Confirm::with_theme(&self.theme)
            .with_prompt("\nProceed with deployment?")
            .default(true)
            .interact()?;

        if confirm {
            println!("\n{} Deploying to {}...", "⏳".bright_blue(), network_name.bright_cyan());

            // Use existing deploy command function
            match commands::deploy::execute(network_name.to_string(), force, None).await {
                Ok(_) => {
                    println!("{} {} {}", "✅".bright_green(), "Successfully deployed to".bright_green(), network_name.bright_cyan().bold());
                    println!("\n{} Next steps:", "💡".bright_yellow());
                    println!("  {} {}", "1.".bright_white(), "icarus mcp start --canister-id <your-canister-id>".bright_cyan());
                    println!("  {} {}", "2.".bright_white(), "Test your MCP server with Claude Desktop".bright_cyan());
                }
                Err(e) => {
                    println!("{} Deployment failed: {}", "❌".bright_red(), e);
                }
            }
        } else {
            println!("{} Deployment cancelled.", "⚠️".bright_yellow());
        }

        Ok(())
    }

    /// Wizard for configuration
    async fn wizard_configure(&self) -> Result<()> {
        println!("\n{} {}", "⚙️".bright_blue(), "Configuration Wizard".bright_cyan().bold());
        println!("{}", "Let's configure your Icarus project settings!\n".bright_white());

        // Check if we're in a project directory
        if !Path::new("Cargo.toml").exists() {
            println!("{} {} {}",
                "⚠️".bright_yellow(),
                "No Cargo.toml found in current directory.".bright_yellow(),
                "Are you in an Icarus project?".bright_white()
            );
            return Ok(());
        }

        // Load existing configuration
        let config = match IcarusConfig::load() {
            Ok(config) => {
                println!("{} Loaded existing configuration", "✅".bright_green());
                Some(config)
            }
            Err(_) => {
                println!("{} No existing configuration found, will create new one", "💡".bright_yellow());
                None
            }
        };

        // Configuration options
        let config_actions = vec![
            "🔧 Basic project settings",
            "🌐 Network configuration",
            "🔐 Identity and authentication",
            "📊 Monitoring and logging",
            "💾 Save and exit",
        ];

        loop {
            let selection = Select::with_theme(&self.theme)
                .with_prompt("What would you like to configure?")
                .items(&config_actions)
                .default(0)
                .interact()?;

            match selection {
                0 => self.configure_basic_settings(&config).await?,
                1 => self.configure_network_settings(&config).await?,
                2 => self.configure_identity_settings(&config).await?,
                3 => self.configure_monitoring_settings(&config).await?,
                4 => {
                    println!("{} Configuration completed!", "✅".bright_green());
                    break;
                }
                _ => unreachable!(),
            }
        }

        Ok(())
    }

    /// Basic project settings configuration
    async fn configure_basic_settings(&self, _config: &Option<IcarusConfig>) -> Result<()> {
        println!("\n{} {}", "🔧".bright_blue(), "Basic Settings".bright_cyan().bold());

        // For now, just show a placeholder
        println!("{} Basic settings configuration coming soon!", "🚧".bright_yellow());

        Ok(())
    }

    /// Network settings configuration
    async fn configure_network_settings(&self, _config: &Option<IcarusConfig>) -> Result<()> {
        println!("\n{} {}", "🌐".bright_blue(), "Network Settings".bright_cyan().bold());

        // For now, just show a placeholder
        println!("{} Network settings configuration coming soon!", "🚧".bright_yellow());

        Ok(())
    }

    /// Identity settings configuration
    async fn configure_identity_settings(&self, _config: &Option<IcarusConfig>) -> Result<()> {
        println!("\n{} {}", "🔐".bright_blue(), "Identity Settings".bright_cyan().bold());

        // For now, just show a placeholder
        println!("{} Identity settings configuration coming soon!", "🚧".bright_yellow());

        Ok(())
    }

    /// Monitoring settings configuration
    async fn configure_monitoring_settings(&self, _config: &Option<IcarusConfig>) -> Result<()> {
        println!("\n{} {}", "📊".bright_blue(), "Monitoring Settings".bright_cyan().bold());

        // For now, just show a placeholder
        println!("{} Monitoring settings configuration coming soon!", "🚧".bright_yellow());

        Ok(())
    }

    /// Wizard for project analysis
    async fn wizard_analyze(&self) -> Result<()> {
        println!("\n{} {}", "🔍".bright_blue(), "Project Analysis Wizard".bright_cyan().bold());
        println!("{}", "Let's analyze your Icarus project!\n".bright_white());

        // Check if we're in a project directory
        if !Path::new("Cargo.toml").exists() {
            println!("{} {} {}",
                "⚠️".bright_yellow(),
                "No Cargo.toml found in current directory.".bright_yellow(),
                "Are you in an Icarus project?".bright_white()
            );
            return Ok(());
        }

        let analysis_options = vec![
            "📊 WASM binary size analysis",
            "🏗️  Project structure overview",
            "🔧 Dependencies analysis",
            "⚡ Performance recommendations",
        ];

        let selection = Select::with_theme(&self.theme)
            .with_prompt("What would you like to analyze?")
            .items(&analysis_options)
            .default(0)
            .interact()?;

        match selection {
            0 => {
                println!("\n{} Running WASM binary analysis...", "⏳".bright_blue());
                // Use existing analyze command function
                if let Err(e) = commands::analyze::execute(20, true).await {
                    println!("{} Analysis failed: {}", "❌".bright_red(), e);
                }
            }
            1 => {
                self.analyze_project_structure().await?;
            }
            2 => {
                self.analyze_dependencies().await?;
            }
            3 => {
                self.analyze_performance().await?;
            }
            _ => unreachable!(),
        }

        Ok(())
    }

    /// Analyze project structure
    async fn analyze_project_structure(&self) -> Result<()> {
        println!("\n{} {}", "🏗️".bright_blue(), "Project Structure Analysis".bright_cyan().bold());

        // Basic project structure analysis
        let paths_to_check = vec![
            ("src/lib.rs", "📄 Main library file"),
            ("Cargo.toml", "📦 Package manifest"),
            ("canister.toml", "🏺 Canister configuration"),
            ("tests/", "🧪 Test directory"),
            ("README.md", "📖 Documentation"),
        ];

        println!("\n{} Project Files:", "📁".bright_cyan());
        for (path, description) in paths_to_check {
            let exists = Path::new(path).exists();
            let status = if exists { "✅".bright_green() } else { "❌".bright_red() };
            println!("  {} {} - {}", status, path.bright_white(), description);
        }

        Ok(())
    }

    /// Analyze dependencies
    async fn analyze_dependencies(&self) -> Result<()> {
        println!("\n{} {}", "🔧".bright_blue(), "Dependencies Analysis".bright_cyan().bold());
        println!("{} Dependencies analysis coming soon!", "🚧".bright_yellow());
        Ok(())
    }

    /// Analyze performance
    async fn analyze_performance(&self) -> Result<()> {
        println!("\n{} {}", "⚡".bright_blue(), "Performance Analysis".bright_cyan().bold());
        println!("{} Performance analysis coming soon!", "🚧".bright_yellow());
        Ok(())
    }

    /// Health check wizard
    async fn wizard_health_check(&self) -> Result<()> {
        println!("\n{} {}", "🏥".bright_blue(), "Health Check & Diagnostics".bright_cyan().bold());
        println!("{}", "Let's check the health of your development environment!\n".bright_white());

        println!("{} Running diagnostics...", "⏳".bright_blue());

        // Check dfx installation
        self.check_dfx_installation().await?;

        // Check Internet Computer connection
        self.check_ic_connection().await?;

        // Check project health (if in project directory)
        if Path::new("Cargo.toml").exists() {
            self.check_project_health().await?;
        }

        println!("\n{} Health check completed!", "✅".bright_green());

        Ok(())
    }

    /// Check dfx installation
    async fn check_dfx_installation(&self) -> Result<()> {
        println!("\n{} Checking dfx installation...", "🔍".bright_blue());

        match tokio::process::Command::new("dfx")
            .args(&["--version"])
            .output()
            .await
        {
            Ok(output) if output.status.success() => {
                let version = String::from_utf8_lossy(&output.stdout);
                println!("  {} dfx found: {}", "✅".bright_green(), version.trim().bright_cyan());
            }
            _ => {
                println!("  {} dfx not found or not working", "❌".bright_red());
                println!("    💡 Install dfx: https://internetcomputer.org/docs/current/developer-docs/setup/install");
            }
        }

        Ok(())
    }

    /// Check IC connection
    async fn check_ic_connection(&self) -> Result<()> {
        println!("\n{} Checking Internet Computer connection...", "🔍".bright_blue());

        // Check if dfx is running locally
        match tokio::process::Command::new("dfx")
            .args(&["ping", "local"])
            .output()
            .await
        {
            Ok(output) if output.status.success() => {
                println!("  {} Local IC replica is running", "✅".bright_green());
            }
            _ => {
                println!("  {} Local IC replica not running", "⚠️".bright_yellow());
                println!("    💡 Start with: dfx start --background");
            }
        }

        Ok(())
    }

    /// Check project health
    async fn check_project_health(&self) -> Result<()> {
        println!("\n{} Checking project health...", "🔍".bright_blue());

        // Check if project builds
        match tokio::process::Command::new("cargo")
            .args(&["check", "--quiet"])
            .output()
            .await
        {
            Ok(output) if output.status.success() => {
                println!("  {} Project builds successfully", "✅".bright_green());
            }
            Ok(output) => {
                println!("  {} Project has build issues", "❌".bright_red());
                let stderr = String::from_utf8_lossy(&output.stderr);
                if !stderr.is_empty() {
                    println!("    {}", stderr.trim());
                }
            }
            Err(_) => {
                println!("  {} Could not check project build", "⚠️".bright_yellow());
            }
        }

        Ok(())
    }

    /// Show welcome banner
    fn show_welcome_banner(&self) {
        println!("\n{}", "╔═══════════════════════════════════════════════════════════════╗".bright_cyan());
        println!("{}", "║                                                               ║".bright_cyan());
        println!("{}", "║  🧙✨  ICARUS INTERACTIVE WIZARD  ✨🧙                      ║".bright_cyan());
        println!("{}", "║                                                               ║".bright_cyan());
        println!("{}", "║  Welcome to the guided setup and management experience!      ║".bright_cyan());
        println!("{}", "║  This wizard will help you with common Icarus operations.    ║".bright_cyan());
        println!("{}", "║                                                               ║".bright_cyan());
        println!("{}", "╚═══════════════════════════════════════════════════════════════╝".bright_cyan());
        println!();
    }

    /// Show quick system status
    async fn show_quick_status(&self) {
        println!("{} {}", "📊".bright_blue(), "Quick System Status".bright_cyan().bold());
        println!("{}", "────────────────────────────────".bright_blue());

        // Check if we're in a project
        let in_project = Path::new("Cargo.toml").exists() && Path::new("dfx.json").exists();
        let project_status = if in_project { "✅ Icarus Project".bright_green() } else { "📁 No Project".bright_yellow() };
        println!("  Project: {}", project_status);

        // Check dfx availability
        let dfx_available = tokio::process::Command::new("dfx")
            .args(&["--version"])
            .output()
            .await
            .map(|output| output.status.success())
            .unwrap_or(false);

        let dfx_status = if dfx_available { "✅ Available".bright_green() } else { "❌ Not Found".bright_red() };
        println!("  DFX: {}", dfx_status);

        // Check if local replica is running
        if dfx_available {
            let replica_running = tokio::process::Command::new("dfx")
                .args(&["ping", "local"])
                .output()
                .await
                .map(|output| output.status.success())
                .unwrap_or(false);

            let replica_status = if replica_running { "✅ Running".bright_green() } else { "⏹️ Stopped".bright_yellow() };
            println!("  Local IC: {}", replica_status);
        } else {
            println!("  Local IC: {} Cannot Check", "⚠️".bright_yellow());
        }

        println!();
    }
}