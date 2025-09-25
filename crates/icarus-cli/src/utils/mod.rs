pub mod build_utils;
pub mod claude_desktop;
pub mod dfx;
pub mod mcp_clients;
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

/// Run a command with custom environment variables
pub async fn run_command_with_env(
    program: &str,
    args: &[&str],
    working_dir: Option<&std::path::Path>,
    env_vars: &[(&str, String)],
) -> Result<String> {
    use tokio::process::Command;

    let mut cmd = Command::new(program);
    cmd.args(args);

    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }

    // Set environment variables
    for (key, value) in env_vars {
        cmd.env(key, value);
    }

    let output = cmd.output().await?;

    if !output.status.success() {
        anyhow::bail!(
            "Command failed: {} {} (exit code: {:?})\nstderr: {}",
            program,
            args.join(" "),
            output.status.code(),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Run a command with streaming output (shows real-time progress)
pub async fn run_command_streaming(
    program: &str,
    args: &[&str],
    working_dir: Option<&std::path::Path>,
) -> Result<String> {
    use std::process::Stdio;
    use tokio::io::{AsyncBufReadExt, BufReader};
    use tokio::process::Command;

    let mut cmd = Command::new(program);
    cmd.args(args);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }

    let mut child = cmd.spawn()?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow::anyhow!("Failed to capture stdout"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| anyhow::anyhow!("Failed to capture stderr"))?;

    let mut stdout_lines = BufReader::new(stdout).lines();
    let mut stderr_lines = BufReader::new(stderr).lines();

    let mut output_buffer = String::new();

    // Read and display output in real-time
    tokio::select! {
        _ = async {
            while let Ok(Some(line)) = stdout_lines.next_line().await {
                println!("{}", line);
                output_buffer.push_str(&line);
                output_buffer.push('\n');
            }
        } => {},
        _ = async {
            while let Ok(Some(line)) = stderr_lines.next_line().await {
                eprintln!("{}", line);
            }
        } => {}
    }

    let status = child.wait().await?;

    if !status.success() {
        anyhow::bail!(
            "Command failed: {} {} (exit code: {:?})",
            program,
            args.join(" "),
            status.code()
        );
    }

    Ok(output_buffer)
}

pub fn ensure_directory_exists(path: &std::path::Path) -> Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}
