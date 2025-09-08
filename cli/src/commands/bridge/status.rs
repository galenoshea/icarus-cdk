use anyhow::Result;
use colored::Colorize;
use sysinfo::{ProcessesToUpdate, System};

use crate::utils::{print_info, print_success, print_warning};

#[derive(Debug)]
struct BridgeInstance {
    port: u16,
    pid: u32,
    canister_id: Option<String>,
    memory_mb: f64,
    cpu_usage: f32,
}

pub async fn execute(verbose: bool) -> Result<()> {
    print_info("Checking Icarus bridge status...");

    let instances = find_bridge_instances()?;

    if instances.is_empty() {
        print_warning("No Icarus bridge instances found");
        println!("\nStart a bridge with: icarus bridge start --canister-id <ID>");
        return Ok(());
    }

    print_success(&format!("Found {} bridge instance(s)", instances.len()));

    for instance in instances {
        println!(
            "\n{}",
            format!("Bridge on port {}", instance.port).cyan().bold()
        );
        println!("  PID: {}", instance.pid);

        if let Some(canister_id) = &instance.canister_id {
            println!("  Canister: {}", canister_id);
        }

        if verbose {
            println!("  Memory: {:.1} MB", instance.memory_mb);
            println!("  CPU: {:.1}%", instance.cpu_usage);
        }

        // Check if port is accessible
        if check_bridge_health(instance.port).await {
            println!("  Status: {}", "Running".green());
        } else {
            println!("  Status: {}", "Not responding".red());
        }
    }

    if !verbose {
        println!("\nUse --verbose for detailed information");
    }

    Ok(())
}

fn find_bridge_instances() -> Result<Vec<BridgeInstance>> {
    let mut instances = Vec::new();
    let mut system = System::new_all();
    system.refresh_processes(ProcessesToUpdate::All, true);

    // Look for icarus-bridge processes
    for (pid, process) in system.processes() {
        let name = process.name();
        if name == "icarus-bridge" || name == "icarus-bridge.exe" {
            let cmd_str = process
                .cmd()
                .iter()
                .map(|s| s.to_string_lossy())
                .collect::<Vec<_>>()
                .join(" ");

            // Parse port from command line
            let port = parse_port_from_cmd(&cmd_str);
            let canister_id = parse_canister_from_cmd(&cmd_str);

            if let Some(port) = port {
                instances.push(BridgeInstance {
                    port,
                    pid: pid.as_u32(),
                    canister_id,
                    memory_mb: process.memory() as f64 / 1_048_576.0,
                    cpu_usage: process.cpu_usage(),
                });
            }
        }
    }

    // Also check PID files
    let config_dir = crate::config::IcarusConfig::config_dir()?;
    if config_dir.exists() {
        for entry in std::fs::read_dir(&config_dir)? {
            let entry = entry?;
            let path = entry.path();

            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with("bridge-") && name.ends_with(".pid") {
                    // Extract port from filename
                    if let Some(port_str) = name
                        .strip_prefix("bridge-")
                        .and_then(|s| s.strip_suffix(".pid"))
                    {
                        if let Ok(port) = port_str.parse::<u16>() {
                            // Read PID from file
                            if let Ok(pid_str) = std::fs::read_to_string(&path) {
                                if let Ok(pid) = pid_str.trim().parse::<u32>() {
                                    // Check if process is still running
                                    if is_process_running(pid) {
                                        // If we don't already have this instance, add it
                                        if !instances.iter().any(|i| i.pid == pid) {
                                            instances.push(BridgeInstance {
                                                port,
                                                pid,
                                                canister_id: None,
                                                memory_mb: 0.0,
                                                cpu_usage: 0.0,
                                            });
                                        }
                                    } else {
                                        // Clean up stale PID file
                                        let _ = std::fs::remove_file(&path);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    instances.sort_by_key(|i| i.port);
    Ok(instances)
}

fn parse_port_from_cmd(cmd: &str) -> Option<u16> {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    for (i, part) in parts.iter().enumerate() {
        if (*part == "--port" || *part == "-p") && i + 1 < parts.len() {
            return parts[i + 1].parse().ok();
        }
    }
    None
}

fn parse_canister_from_cmd(cmd: &str) -> Option<String> {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    for (i, part) in parts.iter().enumerate() {
        if *part == "--canister-id" && i + 1 < parts.len() {
            return Some(parts[i + 1].to_string());
        }
    }
    None
}

fn is_process_running(pid: u32) -> bool {
    let mut system = System::new();
    system.refresh_processes(ProcessesToUpdate::All, true);
    system.process(sysinfo::Pid::from_u32(pid)).is_some()
}

async fn check_bridge_health(port: u16) -> bool {
    // Try to connect to the WebSocket port
    tokio::time::timeout(
        std::time::Duration::from_secs(2),
        tokio::net::TcpStream::connect(format!("127.0.0.1:{}", port)),
    )
    .await
    .is_ok()
}
