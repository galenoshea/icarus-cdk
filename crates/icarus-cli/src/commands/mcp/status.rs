use anyhow::Result;
use colored::Colorize;
use sysinfo::{ProcessesToUpdate, System};

use crate::utils::{print_info, print_success, print_warning};

#[derive(Debug)]
struct McpInstance {
    pid: u32,
    canister_id: Option<String>,
    memory_mb: f64,
    cpu_usage: f32,
    command_line: String,
}

pub async fn execute(verbose: bool) -> Result<()> {
    print_info("Checking Icarus MCP server status...");

    let instances = find_mcp_instances()?;

    if instances.is_empty() {
        print_warning("No Icarus MCP server instances found");
        println!("\nStart an MCP server with: icarus mcp start --canister-id <ID>");
        return Ok(());
    }

    print_success(&format!("Found {} MCP server instance(s)", instances.len()));
    let instance_count = instances.len();

    for instance in instances {
        println!(
            "\n{}",
            format!("MCP Server (PID: {})", instance.pid).cyan().bold()
        );

        if let Some(canister_id) = &instance.canister_id {
            println!("  Canister: {}", canister_id);
        }

        if verbose {
            println!("  Memory: {:.1} MB", instance.memory_mb);
            println!("  CPU: {:.1}%", instance.cpu_usage);
            println!("  Command: {}", instance.command_line);
        }

        // Since MCP servers typically run via stdio, we can't do a simple port check
        // Instead, we just confirm the process is running
        println!("  Status: {}", "Running".green());
        println!("  Protocol: MCP over stdio");
    }

    if instance_count == 1 {
        println!("\n{}", "Connection info:".bold());
        println!("  The MCP server communicates via stdin/stdout");
        println!("  Configure your MCP client to launch: icarus mcp start --canister-id <ID>");
    }

    Ok(())
}

fn find_mcp_instances() -> Result<Vec<McpInstance>> {
    let mut system = System::new_all();
    system.refresh_processes(ProcessesToUpdate::All, true);

    let mut instances = Vec::new();

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
            // Extract canister ID from command line
            let canister_id = extract_canister_id(&cmd_strings);

            let instance = McpInstance {
                pid: process.pid().as_u32(),
                canister_id,
                memory_mb: process.memory() as f64 / 1024.0 / 1024.0,
                cpu_usage: process.cpu_usage(),
                command_line: cmd_strings.join(" "),
            };

            instances.push(instance);
        }
    }

    Ok(instances)
}

fn extract_canister_id(cmd_line: &[String]) -> Option<String> {
    // Look for --canister-id argument
    for (i, arg) in cmd_line.iter().enumerate() {
        if arg == "--canister-id" && i + 1 < cmd_line.len() {
            return Some(cmd_line[i + 1].clone());
        }
    }
    None
}
