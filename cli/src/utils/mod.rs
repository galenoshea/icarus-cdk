pub mod claude_desktop;
pub mod dfx;
pub mod platform;

use anyhow::Result;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

pub fn create_progress_bar(len: u64, message: &str) -> ProgressBar {
    let pb = ProgressBar::new(len);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} {msg} [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
            .unwrap()
            .progress_chars("#>-"),
    );
    pb.set_message(message.to_string());
    pb
}

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

pub fn ensure_directory_exists(path: &std::path::Path) -> Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}
