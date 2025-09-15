//! Beautiful UI helpers for enhanced terminal experience

use anyhow::Result;
use colored::Colorize;
use crossterm::terminal;
use dialoguer::{theme::ColorfulTheme, Confirm, MultiSelect};
use indicatif::{ProgressBar, ProgressStyle};
use std::{io::Write, time::Duration};
use unicode_width::UnicodeWidthStr;

use super::mcp_clients::{ClientInfo, ClientType};

/// Create a beautiful themed interface
pub fn create_theme() -> ColorfulTheme {
    ColorfulTheme::default()
}

/// Create a beautiful spinner with custom message
pub fn create_beautiful_spinner(message: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"),
    );
    pb.set_message(message.to_string());
    pb.enable_steady_tick(Duration::from_millis(120));
    pb
}

/// Display a beautiful header with borders
pub fn display_header(title: &str) {
    let term_width = terminal::size().map(|(w, _)| w as usize).unwrap_or(80);
    let title_width = title.width();
    let border_width = term_width.saturating_sub(4);
    let padding = border_width.saturating_sub(title_width) / 2;

    println!();
    println!("{}", "━".repeat(border_width).cyan());
    println!(
        "{}{}{}",
        " ".repeat(padding),
        title.bold().cyan(),
        " ".repeat(border_width.saturating_sub(padding + title_width))
    );
    println!("{}", "━".repeat(border_width).cyan());
    println!();
}

/// Display a beautiful section header
pub fn display_section(title: &str) {
    println!();
    println!("{} {}", "📋".cyan(), title.bold());
    println!("{}", "─".repeat(title.width() + 3).cyan());
}

/// Create interactive client selection with rich UI
pub fn select_clients_interactive_beautiful(client_infos: &[&ClientInfo]) -> Result<Vec<usize>> {
    if client_infos.is_empty() {
        println!("{} No AI clients detected on this system.", "⚠️".yellow());
        return Ok(vec![]);
    }

    display_header("🚀 Icarus MCP Configuration");

    // Create rich options with status information
    let options: Vec<String> = client_infos
        .iter()
        .map(|info| {
            let server_count = 0; // We'll need to fetch this from the client
            let status_color = if info.is_installed {
                if server_count > 0 {
                    "green"
                } else {
                    "yellow"
                }
            } else {
                "red"
            };

            let status_text = if info.is_installed {
                if server_count > 0 {
                    format!("({} servers)", server_count)
                } else {
                    "(no servers)".to_string()
                }
            } else {
                "(not installed)".to_string()
            };

            format!(
                "{} {} {}",
                info.client_type.emoji(),
                info.client_type.display_name(),
                match status_color {
                    "green" => status_text.green(),
                    "yellow" => status_text.yellow(),
                    "red" => status_text.red(),
                    _ => status_text.normal(),
                }
            )
        })
        .collect();

    let theme = create_theme();

    if client_infos.len() == 1 {
        let _client = &client_infos[0];
        println!("Found {}", options[0]);
        println!();

        let confirmed = Confirm::with_theme(&theme)
            .with_prompt("Configure this client?")
            .default(true)
            .interact()?;

        if confirmed {
            Ok(vec![0])
        } else {
            Ok(vec![])
        }
    } else {
        println!("Select AI clients to configure:");
        println!(
            "{}",
            "Use Space to select, Enter to confirm, A for all".dimmed()
        );
        println!();

        let selections = MultiSelect::with_theme(&theme).items(&options).interact()?;

        Ok(selections)
    }
}

/// Display a beautiful confirmation dialog
pub fn confirm_beautiful(message: &str) -> Result<bool> {
    let theme = create_theme();

    println!();
    Confirm::with_theme(&theme)
        .with_prompt(message)
        .default(true)
        .interact()
        .map_err(Into::into)
}

/// Display success message with animation
pub fn display_success_animated(message: &str) {
    print!("🎉 ");
    std::io::stdout().flush().unwrap();
    std::thread::sleep(Duration::from_millis(200));

    for char in message.chars() {
        print!("{}", char.to_string().green().bold());
        std::io::stdout().flush().unwrap();
        std::thread::sleep(Duration::from_millis(30));
    }
    println!();
}

/// Display error message with appropriate styling
pub fn display_error_styled(message: &str) {
    println!("{} {}", "❌".red(), message.red().bold());
}

/// Display warning message with appropriate styling
pub fn display_warning_styled(message: &str) {
    println!("{} {}", "⚠️".yellow(), message.yellow().bold());
}

/// Display info message with appropriate styling
pub fn display_info_styled(message: &str) {
    println!("{} {}", "ℹ️".blue(), message);
}

/// Display a progress bar for batch operations
pub fn create_progress_bar(len: u64, message: &str) -> ProgressBar {
    let pb = ProgressBar::new(len);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("█▉▊▋▌▍▎▏  "),
    );
    pb.set_message(message.to_string());
    pb
}

/// Display a tree-like structure for configuration listing
pub fn display_config_tree(client_infos: &[(ClientType, bool, Vec<String>)]) {
    display_header("🌳 MCP Server Configuration Tree");

    for (i, (client_type, is_installed, servers)) in client_infos.iter().enumerate() {
        let is_last = i == client_infos.len() - 1;
        let tree_char = if is_last { "└──" } else { "├──" };

        println!(
            "{} {} {}",
            tree_char.cyan(),
            client_type.emoji(),
            if *is_installed {
                client_type.display_name().bold()
            } else {
                client_type.display_name().dimmed()
            }
        );

        for (j, server) in servers.iter().enumerate() {
            let is_last_server = j == servers.len() - 1;
            let prefix = if is_last { "    " } else { "│   " };
            let server_char = if is_last_server {
                "└── "
            } else {
                "├── "
            };

            println!(
                "{}{}{} {}",
                prefix,
                server_char.dimmed(),
                "🚀".cyan(),
                server.green()
            );
        }

        if servers.is_empty() {
            let prefix = if is_last { "    " } else { "│   " };
            println!("{}└── {}", prefix, "(no servers configured)".dimmed());
        }

        if !is_last {
            println!("│");
        }
    }

    println!();
}

#[cfg(test)]
mod tests {}
