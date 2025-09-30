use anyhow::{anyhow, Result};
use colored::Colorize;
use tokio::fs;
use tracing::info;

use crate::{commands::mcp::StopArgs, Cli};

pub(crate) async fn execute(args: StopArgs, cli: &Cli) -> Result<()> {
    info!("Stopping MCP bridge server");

    if !cli.quiet {
        println!("{} Stopping MCP bridge server", "→".bright_blue());
    }

    if args.all {
        stop_all_processes(args.force, cli).await
    } else {
        stop_daemon_process(args.force, cli).await
    }
}

async fn stop_daemon_process(force: bool, cli: &Cli) -> Result<()> {
    let pid_file = "/tmp/icarus-mcp-bridge.pid";

    // Check if PID file exists
    if !std::path::Path::new(pid_file).exists() {
        if !cli.quiet {
            println!("{}", "No running MCP bridge daemon found.".yellow());
        }
        return Ok(());
    }

    // Read PID from file
    let pid_str = fs::read_to_string(pid_file).await?;
    let pid: u32 = pid_str
        .trim()
        .parse()
        .map_err(|_| anyhow!("Invalid PID in file: {}", pid_str))?;

    if !cli.quiet {
        println!(
            "  {} Stopping daemon (PID: {})",
            "→".bright_blue(),
            pid.to_string().bright_cyan()
        );
    }

    // Check if process is actually running
    if !is_process_running(pid) {
        if !cli.quiet {
            println!(
                "{}",
                "Process is not running, cleaning up PID file.".yellow()
            );
        }
        fs::remove_file(pid_file).await?;
        return Ok(());
    }

    // Stop the process
    stop_process(pid, force)?;

    // Wait for process to stop
    let mut attempts = 0;
    while is_process_running(pid) && attempts < 30 {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        attempts += 1;
    }

    if is_process_running(pid) {
        if force {
            // Force kill if still running
            kill_process(pid)?;
            if !cli.quiet {
                println!("{} Process forcefully terminated", "⚠️".yellow());
            }
        } else {
            return Err(anyhow!(
                "Process {} did not stop gracefully. Use --force to kill it.",
                pid
            ));
        }
    }

    // Clean up PID file
    fs::remove_file(pid_file).await?;

    if !cli.quiet {
        println!("{} MCP bridge server stopped", "✅".green());
    }

    info!("MCP bridge daemon stopped successfully");
    Ok(())
}

async fn stop_all_processes(force: bool, cli: &Cli) -> Result<()> {
    if !cli.quiet {
        println!("  {} Stopping all MCP bridge processes", "→".bright_blue());
    }

    // Find all icarus MCP processes
    let processes = find_icarus_mcp_processes().await?;

    if processes.is_empty() {
        if !cli.quiet {
            println!("{}", "No running MCP bridge processes found.".yellow());
        }
        return Ok(());
    }

    let mut stopped_count = 0;
    for pid in processes {
        if !cli.quiet {
            println!(
                "  {} Stopping process {}",
                "→".bright_blue(),
                pid.to_string().bright_cyan()
            );
        }

        if stop_process(pid, force).is_ok() {
            // Wait a bit for graceful shutdown
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

            if is_process_running(pid) && force {
                kill_process(pid)?;
            }

            if !is_process_running(pid) {
                stopped_count += 1;
            }
        }
    }

    // Clean up PID file if it exists
    let _ = fs::remove_file("/tmp/icarus-mcp-bridge.pid").await;

    if !cli.quiet {
        println!(
            "{} Stopped {} MCP bridge processes",
            "✅".green(),
            stopped_count.to_string().bright_cyan()
        );
    }

    info!("Stopped {} MCP bridge processes", stopped_count);
    Ok(())
}

fn is_process_running(pid: u32) -> bool {
    #[cfg(unix)]
    {
        use nix::sys::signal::kill;
        use nix::unistd::Pid;

        match kill(Pid::from_raw(pid as i32), None) {
            Ok(()) => true,  // Process exists
            Err(_) => false, // Process doesn't exist
        }
    }

    #[cfg(windows)]
    {
        use windows::Win32::System::Diagnostics::Debug::GetProcessId;
        use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION};

        unsafe {
            let handle = OpenProcess(PROCESS_QUERY_INFORMATION, false, pid);
            match handle {
                Ok(h) => {
                    let _ = h.close();
                    true
                }
                Err(_) => false,
            }
        }
    }

    #[cfg(not(any(unix, windows)))]
    {
        // Fallback for other platforms
        false
    }
}

fn stop_process(pid: u32, force: bool) -> Result<()> {
    #[cfg(unix)]
    {
        use nix::sys::signal::{kill, Signal};
        use nix::unistd::Pid;

        let signal = if force {
            Signal::SIGKILL
        } else {
            Signal::SIGTERM
        };
        kill(Pid::from_raw(pid as i32), signal)
            .map_err(|e| anyhow!("Failed to stop process {}: {}", pid, e))?;
    }

    #[cfg(windows)]
    {
        use windows::Win32::System::Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE};

        unsafe {
            let handle = OpenProcess(PROCESS_TERMINATE, false, pid)
                .map_err(|e| anyhow!("Failed to open process {}: {:?}", pid, e))?;

            let exit_code = if force { 1 } else { 0 };
            TerminateProcess(&handle, exit_code)
                .map_err(|e| anyhow!("Failed to terminate process {}: {:?}", pid, e))?;

            let _ = handle.close();
        }
    }

    #[cfg(not(any(unix, windows)))]
    {
        return Err(anyhow!(
            "Process termination not supported on this platform"
        ));
    }

    Ok(())
}

fn kill_process(pid: u32) -> Result<()> {
    stop_process(pid, true)
}

async fn find_icarus_mcp_processes() -> Result<Vec<u32>> {
    let mut processes = Vec::new();

    #[cfg(unix)]
    {
        use tokio::process::Command;

        // Use ps to find icarus MCP processes
        let output = Command::new("ps")
            .args(["-eo", "pid,comm"])
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.contains("icarus") && line.contains("mcp") {
                if let Some(pid_str) = line.split_whitespace().next() {
                    if let Ok(pid) = pid_str.parse::<u32>() {
                        processes.push(pid);
                    }
                }
            }
        }
    }

    #[cfg(windows)]
    {
        use tokio::process::Command;

        // Use tasklist to find icarus processes
        let output = Command::new("tasklist")
            .args(["/FI", "IMAGENAME eq icarus.exe"])
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.contains("icarus.exe") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(pid) = parts[1].parse::<u32>() {
                        processes.push(pid);
                    }
                }
            }
        }
    }

    Ok(processes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process;

    #[test]
    fn test_process_running_check() {
        // Test with current process (should be running)
        let current_pid = process::id();
        assert!(is_process_running(current_pid));

        // Test with a PID that definitely doesn't exist (very high number)
        assert!(!is_process_running(99_999_999));
    }

    #[tokio::test]
    async fn test_find_processes() {
        // This test will find processes, but we can't guarantee specific results
        let result = find_icarus_mcp_processes().await;
        assert!(result.is_ok());

        let processes = result.unwrap();
        // The result could be empty if no icarus processes are running, which is fine
        assert!(processes.len() >= 0);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_stop_nonexistent_daemon() {
        // Clean up leftover test PID file if it contains test data
        if let Ok(content) = tokio::fs::read_to_string("/tmp/icarus-mcp-bridge.pid").await {
            if content.trim() == "12345" {
                let _ = tokio::fs::remove_file("/tmp/icarus-mcp-bridge.pid").await;
            }
        }

        let args = StopArgs {
            force: false,
            all: false,
        };

        let cli = crate::Cli {
            verbose: false,
            quiet: true,
            force: false,
            command: crate::Commands::Mcp(crate::commands::McpArgs::Stop(args.clone())),
        };

        // Should not error when no daemon is running
        let result = execute(args, &cli).await;
        assert!(result.is_ok());
    }
}

// Additional platform-specific dependencies for process management

#[cfg(windows)]
use windows;
