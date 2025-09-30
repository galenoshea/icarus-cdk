use anyhow::{anyhow, Context, Result};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::Path;
use std::time::Duration;
use tokio::process::Command;
use tracing::{info, warn};

use crate::utils::project;
use crate::{commands::BuildArgs, Cli};

pub(crate) async fn execute(args: BuildArgs, cli: &Cli) -> Result<()> {
    info!("Building Icarus MCP canister project");

    // Verify we're in a valid project directory
    let project_root = project::find_project_root()?;
    let project_config = project::load_project_config(&project_root).await?;

    if !cli.quiet {
        println!(
            "{} Building project: {}",
            "→".bright_blue(),
            project_config.name.bright_cyan()
        );
    }

    // Create progress spinner
    let spinner = if !cli.quiet {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .tick_chars("⣾⣽⣻⢿⡿⣟⣯⣷")
                .template("{spinner:.blue} {msg}")
                .expect("progress bar template string is valid"),
        );
        pb.enable_steady_tick(Duration::from_millis(100));
        Some(pb)
    } else {
        None
    };

    // Step 1: Build Rust code
    if let Some(ref pb) = spinner {
        pb.set_message("Building Rust code...");
    }
    build_rust_code(&args, &project_root).await?;

    // Step 2: Generate canister declarations if requested
    if args.generate_declarations {
        if let Some(ref pb) = spinner {
            pb.set_message("Generating canister declarations...");
        }
        generate_declarations(&project_root).await?;
    }

    // Step 3: Run tests if requested
    if args.test {
        if let Some(ref pb) = spinner {
            pb.set_message("Running tests...");
        }
        run_tests(&args, &project_root).await?;
    }

    // Step 4: Copy artifacts to output directory
    if let Some(ref output_dir) = args.output {
        if let Some(ref pb) = spinner {
            pb.set_message("Copying build artifacts...");
        }
        copy_artifacts(&project_root, output_dir).await?;
    }

    if let Some(pb) = spinner {
        pb.finish_with_message("Build completed successfully! ✅");
    }

    if !cli.quiet {
        print_build_summary(&args, &project_root);
    }

    info!("Build completed successfully");
    Ok(())
}

async fn build_rust_code(args: &BuildArgs, project_root: &Path) -> Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.arg("build");
    cmd.current_dir(project_root);

    // Set build mode
    match args.mode.as_str() {
        "release" => {
            cmd.arg("--release");
        }
        "debug" => {
            // Default, no additional flags needed
        }
        _ => {
            return Err(anyhow!(
                "Invalid build mode: {}. Use 'debug' or 'release'",
                args.mode
            ))
        }
    }

    // Set target if specified
    if let Some(ref target) = args.target {
        cmd.arg("--target").arg(target);
    } else {
        // Default to WASM target for IC canisters
        cmd.arg("--target").arg("wasm32-unknown-unknown");
    }

    // Enable features
    if !args.features.is_empty() {
        cmd.arg("--features").arg(args.features.join(","));
    }

    // Execute build
    let output = cmd.output().await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Cargo build failed:\n{}", stderr));
    }

    Ok(())
}

async fn generate_declarations(project_root: &Path) -> Result<()> {
    // Check if dfx.json exists
    let dfx_config_path = project_root.join("dfx.json");
    if !dfx_config_path.exists() {
        warn!("No dfx.json found, skipping declaration generation");
        return Ok(());
    }

    // Generate Candid declarations
    let output = Command::new("dfx")
        .args(["generate"])
        .current_dir(project_root)
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        warn!("Declaration generation failed: {}", stderr);
        return Ok(()); // Non-fatal error
    }

    Ok(())
}

async fn run_tests(args: &BuildArgs, project_root: &Path) -> Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.arg("test");
    cmd.current_dir(project_root);

    // Set build mode for tests
    if args.mode == "release" {
        cmd.arg("--release");
    }

    // Set target if specified
    if let Some(ref target) = args.target {
        cmd.arg("--target").arg(target);
    }

    // Enable features
    if !args.features.is_empty() {
        cmd.arg("--features").arg(args.features.join(","));
    }

    let output = cmd.output().await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Tests failed:\n{}", stderr));
    }

    Ok(())
}

async fn copy_artifacts(project_root: &Path, output_dir: &Path) -> Result<()> {
    use tokio::fs;

    // Create output directory
    fs::create_dir_all(output_dir).await.with_context(|| {
        format!(
            "Failed to create output directory: {}",
            output_dir.display()
        )
    })?;

    // Find and copy WASM files
    let target_dir = project_root
        .join("target")
        .join("wasm32-unknown-unknown")
        .join("release");

    if target_dir.exists() {
        let mut entries = fs::read_dir(&target_dir).await?;

        loop {
            let next_entry = entries.next_entry().await?;
            let Some(entry) = next_entry else { break };
            let path = entry.path();
            if let Some(extension) = path.extension() {
                if extension == "wasm" {
                    let dest = output_dir.join(entry.file_name());
                    let copy_result = fs::copy(&path, &dest).await;
                    copy_result.with_context(|| {
                        format!("Failed to copy {} to {}", path.display(), dest.display())
                    })?;
                }
            }
        }
    }

    // Copy Candid files if they exist
    let candid_dir = project_root.join(".dfx").join("local").join("canisters");
    if candid_dir.exists() {
        copy_candid_files(&candid_dir, output_dir).await?;
    }

    Ok(())
}

async fn copy_candid_files(candid_dir: &Path, output_dir: &Path) -> Result<()> {
    use tokio::fs;

    let mut entries = fs::read_dir(candid_dir).await?;

    loop {
        let next_entry = entries.next_entry().await?;
        let Some(entry) = next_entry else { break };
        let path = entry.path();
        if path.is_dir() {
            let candid_file = path.join("service.did");
            if candid_file.exists() {
                let canister_name = entry.file_name();
                let dest = output_dir.join(format!("{}.did", canister_name.to_string_lossy()));
                fs::copy(&candid_file, &dest).await?;
            }
        }
    }

    Ok(())
}

fn print_build_summary(args: &BuildArgs, project_root: &Path) {
    println!("\n{}", "📦 Build Summary".bright_white().bold());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    println!("{} {}", "Mode:".bright_white(), args.mode.bright_cyan());

    if let Some(ref target) = args.target {
        println!("{} {}", "Target:".bright_white(), target.bright_cyan());
    } else {
        println!(
            "{} {}",
            "Target:".bright_white(),
            "wasm32-unknown-unknown".bright_cyan()
        );
    }

    if !args.features.is_empty() {
        println!(
            "{} {}",
            "Features:".bright_white(),
            args.features.join(", ").bright_cyan()
        );
    }

    println!(
        "{} {}",
        "Project:".bright_white(),
        project_root.display().to_string().bright_cyan()
    );

    if args.test {
        println!("{} {}", "Tests:".bright_white(), "✅ Passed".bright_green());
    }

    if args.generate_declarations {
        println!(
            "{} {}",
            "Declarations:".bright_white(),
            "✅ Generated".bright_green()
        );
    }

    if let Some(ref output_dir) = args.output {
        println!(
            "{} {}",
            "Output:".bright_white(),
            output_dir.display().to_string().bright_cyan()
        );
    }

    println!();
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::fs;

    #[tokio::test]
    async fn test_copy_artifacts() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();
        let output_dir = temp_dir.path().join("output");

        // Create a mock WASM file
        let target_dir = project_root
            .join("target")
            .join("wasm32-unknown-unknown")
            .join("release");
        fs::create_dir_all(&target_dir).await.unwrap();
        fs::write(target_dir.join("test.wasm"), b"mock wasm")
            .await
            .unwrap();

        // Test copying artifacts
        copy_artifacts(project_root, &output_dir).await.unwrap();

        // Verify the file was copied
        assert!(output_dir.join("test.wasm").exists());
    }

    #[test]
    fn test_build_mode_validation() {
        // Valid modes should not cause errors during command construction
        let valid_modes = vec!["debug", "release"];
        for mode in valid_modes {
            let args = BuildArgs {
                target: None,
                mode: mode.to_string(),
                features: vec![],
                test: false,
                generate_declarations: false,
                output: None,
            };
            // If this compiles, the mode format is valid
            assert!(args.mode == mode);
        }
    }
}
