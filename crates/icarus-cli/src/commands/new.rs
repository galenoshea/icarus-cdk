use anyhow::{anyhow, Context, Result};
use colored::Colorize;
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{info, warn};

use crate::templates::basic;
use crate::utils::git;
use crate::{commands::NewArgs, Cli};

pub(crate) async fn execute(args: NewArgs, cli: &Cli) -> Result<()> {
    info!("Creating new Icarus MCP canister project: {}", args.name);

    // Validate project name
    validate_project_name(&args.name)?;

    // Determine project path
    let project_path = determine_project_path(&args)?;

    // Check if directory already exists
    if project_path.exists() && project_path.read_dir()?.next().is_some() {
        if !cli.force {
            return Err(anyhow!(
                "Directory '{}' already exists and is not empty. Use --force to overwrite.",
                project_path.display()
            ));
        }
        warn!("Directory exists, will overwrite due to --force flag");
    }

    // Create project directory
    fs::create_dir_all(&project_path).await.with_context(|| {
        format!(
            "Failed to create project directory: {}",
            project_path.display()
        )
    })?;

    if !cli.quiet {
        println!("{}", "  Generating project files...".bright_blue());
    }

    // Generate basic project template
    basic::generate_project(&args.name, &project_path)
        .await
        .with_context(|| "Failed to generate project files")?;

    // Initialize git repository if requested
    if !args.no_git {
        init_git_repository(&project_path, cli).await?;
    }

    // Install dependencies if requested
    if !args.no_install {
        install_dependencies(&project_path, cli).await?;
    }

    // Print success message
    if !cli.quiet {
        print_success_message(&args.name, &project_path);
    }

    info!(
        "Project '{}' created successfully at {}",
        args.name,
        project_path.display()
    );
    Ok(())
}

fn validate_project_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(anyhow!("Project name cannot be empty"));
    }

    if !name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Err(anyhow!(
            "Project name can only contain alphanumeric characters, hyphens, and underscores"
        ));
    }

    if name.starts_with('-') || name.ends_with('-') {
        return Err(anyhow!("Project name cannot start or end with a hyphen"));
    }

    Ok(())
}

fn determine_project_path(args: &NewArgs) -> Result<PathBuf> {
    let base_path = args
        .path
        .as_ref()
        .map(|p| p.to_owned())
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    Ok(base_path.join(&args.name))
}

async fn init_git_repository(project_path: &Path, cli: &Cli) -> Result<()> {
    if !cli.quiet {
        println!("{}", "  Initializing git repository...".bright_blue());
    }

    git::init_repository(project_path).await?;
    git::add_gitignore(project_path).await?;

    info!("Git repository initialized");
    Ok(())
}

async fn install_dependencies(project_path: &Path, cli: &Cli) -> Result<()> {
    if !cli.quiet {
        println!("{}", "  Installing dependencies...".bright_blue());
    }

    // Check if cargo is available
    if which::which("cargo").is_err() {
        warn!("Cargo not found, skipping dependency installation");
        return Ok(());
    }

    // Run cargo check to download and compile dependencies
    let output = tokio::process::Command::new("cargo")
        .arg("check")
        .current_dir(project_path)
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        warn!("Dependency installation encountered issues: {}", stderr);
    } else {
        info!("Dependencies installed successfully");
    }

    Ok(())
}

fn print_success_message(project_name: &str, project_path: &Path) {
    println!(
        "\n{}",
        "🎉 Project created successfully!".bright_green().bold()
    );
    println!();
    println!(
        "{} {}",
        "Project:".bright_white().bold(),
        project_name.bright_cyan()
    );
    println!(
        "{} {}",
        "Location:".bright_white().bold(),
        project_path.display().to_string().bright_cyan()
    );
    println!();
    println!("{}", "Next steps:".bright_white().bold());
    println!(
        "  {} cd {}",
        "1.".bright_yellow(),
        project_name.bright_cyan()
    );
    println!("  {} icarus build", "2.".bright_yellow());
    println!("  {} icarus deploy --network local", "3.".bright_yellow());
    println!();
    println!("{}", "Happy coding! 🚀".bright_green());
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_validate_project_name() {
        assert!(validate_project_name("valid-name").is_ok());
        assert!(validate_project_name("valid_name").is_ok());
        assert!(validate_project_name("validname123").is_ok());

        assert!(validate_project_name("").is_err());
        assert!(validate_project_name("-invalid").is_err());
        assert!(validate_project_name("invalid-").is_err());
        assert!(validate_project_name("invalid@name").is_err());
    }

    #[tokio::test]
    async fn test_determine_project_path() {
        let temp_dir = TempDir::new().unwrap();
        let args = NewArgs {
            name: "test-project".to_string(),
            path: Some(temp_dir.path().to_path_buf()),
            no_git: false,
            no_install: false,
        };

        let project_path = determine_project_path(&args).unwrap();
        assert_eq!(project_path, temp_dir.path().join("test-project"));
    }
}
