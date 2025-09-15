//! Utility functions for icarus-dev crate

use anyhow::Result;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

/// Create a spinner progress bar with the given message
pub fn create_spinner(message: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message(message.to_string());
    pb.enable_steady_tick(Duration::from_millis(100));
    pb
}

/// Print a success message with a green checkmark
pub fn print_success(message: &str) {
    println!("{} {}", "✓".green().bold(), message);
}

/// Print an error message with a red X
pub fn print_error(message: &str) {
    eprintln!("{} {}", "✗".red().bold(), message);
}

/// Print a warning message with a yellow warning symbol
pub fn print_warning(message: &str) {
    println!("{} {}", "⚠".yellow().bold(), message);
}

/// Print an info message with a blue info symbol
pub fn print_info(message: &str) {
    println!("{} {}", "ℹ".blue().bold(), message);
}

/// Run a command and capture its output
pub async fn run_command(
    program: &str,
    args: &[&str],
    working_dir: Option<&std::path::Path>,
) -> Result<String> {
    use tokio::process::Command;

    let mut cmd = Command::new(program);
    cmd.args(args);

    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }

    let output = cmd.output().await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!(
            "Command failed: {} {}\nError: {}",
            program,
            args.join(" "),
            stderr
        );
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Run a command with interactive input/output
/// This allows the command to display output in real-time and accept user input
pub async fn run_command_interactive(
    program: &str,
    args: &[&str],
    working_dir: Option<&std::path::Path>,
) -> Result<()> {
    use tokio::process::Command;

    let mut cmd = Command::new(program);
    cmd.args(args)
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit());

    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }

    let status = cmd.status().await?;

    if !status.success() {
        anyhow::bail!(
            "Command failed: {} {} (exit code: {:?})",
            program,
            args.join(" "),
            status.code()
        );
    }

    Ok(())
}

/// Ensure a directory exists, creating it if necessary
pub fn ensure_directory_exists(path: &std::path::Path) -> Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use tempfile::TempDir;

    #[test]
    fn test_create_spinner() {
        let spinner = create_spinner("Testing");
        assert!(!spinner.is_finished());
        spinner.finish();
        assert!(spinner.is_finished());
    }

    #[test]
    fn test_print_functions() {
        // These functions should not panic and should print to stdout/stderr
        // We can't easily test the output without capturing it, but we can ensure they don't panic
        print_success("Success message");
        print_error("Error message");
        print_warning("Warning message");
        print_info("Info message");
    }

    #[tokio::test]
    async fn test_run_command_success() {
        // Test a simple command that should succeed
        let result = run_command("echo", &["hello"], None).await;
        assert!(result.is_ok());
        assert!(result.unwrap().trim() == "hello");
    }

    #[tokio::test]
    async fn test_run_command_with_working_dir() {
        let temp_dir = TempDir::new().unwrap();
        let result = run_command("pwd", &[], Some(temp_dir.path())).await;

        if result.is_ok() {
            let output = result.unwrap();
            assert!(output.contains(temp_dir.path().to_str().unwrap()));
        }
        // If pwd command fails (like on Windows), that's okay for this test
    }

    #[tokio::test]
    async fn test_run_command_failure() {
        // Test a command that should fail
        let result = run_command("nonexistent_command_12345", &[], None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_run_command_with_args() {
        // Test echo with multiple arguments
        let result = run_command("echo", &["hello", "world"], None).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("hello"));
        assert!(output.contains("world"));
    }

    #[tokio::test]
    async fn test_run_command_interactive_success() {
        // Test that interactive command function doesn't panic
        // We use 'true' command which always succeeds and exits immediately
        let result = run_command_interactive("true", &[], None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_command_interactive_failure() {
        // Test that interactive command properly handles failures
        let result = run_command_interactive("false", &[], None).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_ensure_directory_exists_new() {
        let temp_dir = TempDir::new().unwrap();
        let new_dir = temp_dir.path().join("new_directory");

        // Directory shouldn't exist initially
        assert!(!new_dir.exists());

        // Create it
        let result = ensure_directory_exists(&new_dir);
        assert!(result.is_ok());
        assert!(new_dir.exists());
        assert!(new_dir.is_dir());
    }

    #[test]
    fn test_ensure_directory_exists_existing() {
        let temp_dir = TempDir::new().unwrap();

        // Directory already exists
        assert!(temp_dir.path().exists());

        // Should succeed without error
        let result = ensure_directory_exists(temp_dir.path());
        assert!(result.is_ok());
        assert!(temp_dir.path().exists());
    }

    #[test]
    fn test_ensure_directory_exists_nested() {
        let temp_dir = TempDir::new().unwrap();
        let nested_dir = temp_dir.path().join("level1").join("level2").join("level3");

        // Nested directory shouldn't exist initially
        assert!(!nested_dir.exists());

        // Create it (should create all parent directories)
        let result = ensure_directory_exists(&nested_dir);
        assert!(result.is_ok());
        assert!(nested_dir.exists());
        assert!(nested_dir.is_dir());

        // Verify parent directories were created too
        assert!(temp_dir.path().join("level1").exists());
        assert!(temp_dir.path().join("level1").join("level2").exists());
    }

    #[test]
    fn test_path_operations() {
        // Test that path operations work correctly
        let path = Path::new("/some/test/path");
        assert_eq!(path.file_name().unwrap().to_str().unwrap(), "path");
        assert_eq!(path.parent().unwrap(), Path::new("/some/test"));
    }
}