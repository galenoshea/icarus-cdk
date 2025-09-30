//! Cargo utility functions for building and managing Rust projects
//! These are infrastructure functions that will be used as the CLI expands

#![allow(dead_code)]

use anyhow::{anyhow, Context, Result};
use std::path::Path;
use tokio::process::Command;

/// Check if cargo is available
pub(crate) async fn is_cargo_available() -> bool {
    Command::new("cargo")
        .arg("--version")
        .output()
        .await
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Get cargo version
pub(crate) async fn get_cargo_version() -> Result<String> {
    let output = Command::new("cargo")
        .arg("--version")
        .output()
        .await
        .context("Failed to execute cargo --version")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("cargo --version failed: {}", stderr));
    }

    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();

    Ok(version)
}

/// Build project with cargo
pub(crate) async fn build_project(
    project_path: &Path,
    target: Option<&str>,
    release: bool,
    features: &[String],
) -> Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.arg("build");
    cmd.current_dir(project_path);

    if release {
        cmd.arg("--release");
    }

    if let Some(target_arch) = target {
        cmd.arg("--target").arg(target_arch);
    }

    if !features.is_empty() {
        cmd.arg("--features").arg(features.join(","));
    }

    let output = cmd.output().await.context("Failed to build project")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("cargo build failed: {}", stderr));
    }

    Ok(())
}

/// Run tests with cargo
pub(crate) async fn run_tests(
    project_path: &Path,
    target: Option<&str>,
    release: bool,
    features: &[String],
) -> Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.arg("test");
    cmd.current_dir(project_path);

    if release {
        cmd.arg("--release");
    }

    if let Some(target_arch) = target {
        cmd.arg("--target").arg(target_arch);
    }

    if !features.is_empty() {
        cmd.arg("--features").arg(features.join(","));
    }

    let output = cmd.output().await.context("Failed to run tests")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("cargo test failed: {}", stderr));
    }

    Ok(())
}

/// Check project with cargo
pub(crate) async fn check_project(project_path: &Path, target: Option<&str>) -> Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.arg("check");
    cmd.current_dir(project_path);

    if let Some(target_arch) = target {
        cmd.arg("--target").arg(target_arch);
    }

    let output = cmd.output().await.context("Failed to check project")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("cargo check failed: {}", stderr));
    }

    Ok(())
}

/// Run clippy on project
pub(crate) async fn run_clippy(project_path: &Path, fix: bool) -> Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.arg("clippy");
    cmd.current_dir(project_path);

    if fix {
        cmd.arg("--fix").arg("--allow-dirty");
    }

    let output = cmd.output().await.context("Failed to run clippy")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("cargo clippy failed: {}", stderr));
    }

    Ok(())
}

/// Format code with cargo fmt
pub(crate) async fn format_code(project_path: &Path, check: bool) -> Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.arg("fmt");
    cmd.current_dir(project_path);

    if check {
        cmd.arg("--check");
    }

    let output = cmd.output().await.context("Failed to format code")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("cargo fmt failed: {}", stderr));
    }

    Ok(())
}

/// Clean build artifacts
pub(crate) async fn clean_project(project_path: &Path) -> Result<()> {
    let output = Command::new("cargo")
        .arg("clean")
        .current_dir(project_path)
        .output()
        .await
        .context("Failed to clean project")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("cargo clean failed: {}", stderr));
    }

    Ok(())
}

/// Install cargo target
pub(crate) async fn install_target(target: &str) -> Result<()> {
    let output = Command::new("rustup")
        .args(["target", "add", target])
        .output()
        .await
        .context("Failed to install target")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("rustup target add failed: {}", stderr));
    }

    Ok(())
}

/// Check if target is installed
pub(crate) async fn is_target_installed(target: &str) -> Result<bool> {
    let output = Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()
        .await
        .context("Failed to list installed targets")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("rustup target list failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.lines().any(|line| line.trim() == target))
}

/// Get project dependencies
pub(crate) async fn get_dependencies(project_path: &Path) -> Result<Vec<String>> {
    let output = Command::new("cargo")
        .args(["tree", "--format", "{p}"])
        .current_dir(project_path)
        .output()
        .await
        .context("Failed to get dependencies")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("cargo tree failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let dependencies: Vec<String> = stdout
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect();

    Ok(dependencies)
}

/// Update dependencies
pub(crate) async fn update_dependencies(project_path: &Path) -> Result<()> {
    let output = Command::new("cargo")
        .arg("update")
        .current_dir(project_path)
        .output()
        .await
        .context("Failed to update dependencies")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("cargo update failed: {}", stderr));
    }

    Ok(())
}

/// Generate documentation
pub(crate) async fn generate_docs(project_path: &Path, open: bool) -> Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.arg("doc");
    cmd.current_dir(project_path);

    if open {
        cmd.arg("--open");
    }

    let output = cmd
        .output()
        .await
        .context("Failed to generate documentation")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("cargo doc failed: {}", stderr));
    }

    Ok(())
}

/// Run benchmarks
pub(crate) async fn run_benchmarks(project_path: &Path) -> Result<()> {
    let output = Command::new("cargo")
        .arg("bench")
        .current_dir(project_path)
        .output()
        .await
        .context("Failed to run benchmarks")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("cargo bench failed: {}", stderr));
    }

    Ok(())
}

/// Check for outdated dependencies
pub(crate) async fn check_outdated(project_path: &Path) -> Result<String> {
    let output = Command::new("cargo")
        .arg("outdated")
        .current_dir(project_path)
        .output()
        .await
        .context("Failed to check outdated dependencies")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("cargo outdated failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::fs;

    #[tokio::test]
    async fn test_cargo_availability() {
        let available = is_cargo_available().await;
        println!("cargo available: {}", available);
        // This test just checks that the function runs without error
    }

    #[tokio::test]
    async fn test_cargo_version() {
        if !is_cargo_available().await {
            return; // Skip test if cargo is not available
        }

        let result = get_cargo_version().await;
        if result.is_ok() {
            let version = result.unwrap();
            println!("cargo version: {}", version);
            assert!(version.contains("cargo"));
        }
    }

    #[tokio::test]
    async fn test_target_operations() {
        if !is_cargo_available().await {
            return; // Skip test if cargo is not available
        }

        // Check if wasm32-unknown-unknown target is installed
        let target = "wasm32-unknown-unknown";
        let result = is_target_installed(target).await;

        if result.is_ok() {
            let installed = result.unwrap();
            println!("Target {} installed: {}", target, installed);

            // If not installed, try to install it (this might fail in CI)
            if !installed {
                let install_result = install_target(target).await;
                println!("Install target result: {:?}", install_result);
            }
        }
    }

    async fn create_test_cargo_project(project_path: &Path) -> Result<()> {
        let cargo_content = r#"[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
"#;

        fs::write(project_path.join("Cargo.toml"), cargo_content).await?;

        let src_dir = project_path.join("src");
        fs::create_dir_all(&src_dir).await?;

        let main_content = r#"fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
"#;

        fs::write(src_dir.join("main.rs"), main_content).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_project_operations() {
        if !is_cargo_available().await {
            return; // Skip test if cargo is not available
        }

        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // Create a minimal test project
        if create_test_cargo_project(project_path).await.is_err() {
            return; // Skip if we can't create test project
        }

        // Test cargo check
        let check_result = check_project(project_path, None).await;
        println!("Cargo check result: {:?}", check_result);

        // Test cargo build
        let build_result = build_project(project_path, None, false, &[]).await;
        println!("Cargo build result: {:?}", build_result);

        // Test cargo test
        let test_result = run_tests(project_path, None, false, &[]).await;
        println!("Cargo test result: {:?}", test_result);

        // Test cargo clean
        let clean_result = clean_project(project_path).await;
        println!("Cargo clean result: {:?}", clean_result);
    }

    #[tokio::test]
    async fn test_formatting_and_linting() {
        if !is_cargo_available().await {
            return; // Skip test if cargo is not available
        }

        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        if create_test_cargo_project(project_path).await.is_err() {
            return; // Skip if we can't create test project
        }

        // Test cargo fmt (check mode to avoid modifying files)
        let fmt_result = format_code(project_path, true).await;
        println!("Cargo fmt check result: {:?}", fmt_result);

        // Test cargo clippy (might not be available in all environments)
        let clippy_result = run_clippy(project_path, false).await;
        println!("Cargo clippy result: {:?}", clippy_result);
    }
}
