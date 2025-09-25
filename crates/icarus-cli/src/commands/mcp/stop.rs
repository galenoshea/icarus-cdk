use anyhow::Result;
use sysinfo::{ProcessesToUpdate, Signal, System};

use crate::utils::{print_error, print_info, print_success, print_warning};

pub async fn execute(all: bool, canister_id: Option<String>) -> Result<()> {
    if all {
        stop_all_mcp_servers().await
    } else if let Some(id) = canister_id {
        stop_mcp_server_for_canister(&id).await
    } else {
        print_error("Specify either --all or --canister-id <CANISTER_ID>");
        anyhow::bail!("No target specified");
    }
}

async fn stop_all_mcp_servers() -> Result<()> {
    print_info("Stopping all Icarus MCP server instances...");

    let mut system = System::new_all();
    system.refresh_processes(ProcessesToUpdate::All, true);

    let mut stopped_count = 0;

    for process in system.processes().values() {
        let cmd_line = process.cmd();

        // Convert OsString to String for processing
        let cmd_strings: Vec<String> = cmd_line
            .iter()
            .map(|s| s.to_string_lossy().to_string())
            .collect();

        // Look for both icarus-mcp binary and icarus mcp start commands
        let is_mcp_server = cmd_strings.iter().any(|arg| {
            arg.contains("icarus-mcp")
                || (cmd_strings.len() >= 2
                    && cmd_strings.iter().any(|a| a.contains("icarus"))
                    && cmd_strings.iter().any(|a| a == "mcp"))
        });

        if is_mcp_server {
            if stop_process(
                process.pid().as_u32(),
                &format!("MCP server (PID: {})", process.pid()),
            ) {
                stopped_count += 1;
            }
        }
    }

    if stopped_count > 0 {
        print_success(&format!("Stopped {} MCP server instance(s)", stopped_count));
    } else {
        print_warning("No MCP server instances found to stop");
    }

    Ok(())
}

async fn stop_mcp_server_for_canister(canister_id: &str) -> Result<()> {
    print_info(&format!(
        "Stopping MCP server for canister {}...",
        canister_id
    ));

    let mut system = System::new_all();
    system.refresh_processes(ProcessesToUpdate::All, true);

    let mut stopped_count = 0;

    for process in system.processes().values() {
        let cmd_line = process.cmd();

        // Convert OsString to String for processing
        let cmd_strings: Vec<String> = cmd_line
            .iter()
            .map(|s| s.to_string_lossy().to_string())
            .collect();

        // Check if this is an MCP server for the specific canister
        let is_target_mcp_server = cmd_strings.iter().any(|arg| {
            arg.contains("icarus-mcp")
                || (cmd_strings.len() >= 2
                    && cmd_strings.iter().any(|a| a.contains("icarus"))
                    && cmd_strings.iter().any(|a| a == "mcp"))
        }) && cmd_strings.iter().any(|arg| arg == canister_id);

        if is_target_mcp_server {
            if stop_process(
                process.pid().as_u32(),
                &format!(
                    "MCP server for canister {} (PID: {})",
                    canister_id,
                    process.pid()
                ),
            ) {
                stopped_count += 1;
            }
        }
    }

    if stopped_count > 0 {
        print_success(&format!(
            "Stopped {} MCP server instance(s) for canister {}",
            stopped_count, canister_id
        ));
    } else {
        print_warning(&format!(
            "No MCP server instances found for canister {}",
            canister_id
        ));
    }

    Ok(())
}

fn stop_process(pid: u32, description: &str) -> bool {
    let mut system = System::new();

    if let Some(process) = system.process(sysinfo::Pid::from_u32(pid)) {
        print_info(&format!("Stopping {}...", description));

        // Try SIGTERM first (graceful)
        if process.kill_with(Signal::Term).is_some() {
            // Wait a moment for graceful shutdown
            std::thread::sleep(std::time::Duration::from_secs(2));

            // Check if it's still running
            system.refresh_processes(ProcessesToUpdate::All, true);

            if system.process(sysinfo::Pid::from_u32(pid)).is_none() {
                return true; // Successfully stopped
            }

            // If still running, force kill
            if let Some(process) = system.process(sysinfo::Pid::from_u32(pid)) {
                if process.kill_with(Signal::Kill).is_some() {
                    print_warning(&format!("Force killed {}", description));
                    return true;
                }
            }
        }

        print_error(&format!("Failed to stop {}", description));
        false
    } else {
        print_warning(&format!(
            "Process {} not found (already stopped?)",
            description
        ));
        false
    }
}
