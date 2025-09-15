pub mod claude_desktop;
pub mod dfx;
pub mod mcp_clients;
pub mod platform;
pub mod ui;

use anyhow::Result;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

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

pub fn print_success(message: &str) {
    println!("{} {}", "✓".green().bold(), message);
}

pub fn print_error(message: &str) {
    eprintln!("{} {}", "✗".red().bold(), message);
}

pub fn print_warning(message: &str) {
    println!("{} {}", "⚠".yellow().bold(), message);
}

pub fn print_info(message: &str) {
    println!("{} {}", "ℹ".blue().bold(), message);
}

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

pub fn ensure_directory_exists(path: &std::path::Path) -> Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}
