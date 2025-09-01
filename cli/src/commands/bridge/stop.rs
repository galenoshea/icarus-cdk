use anyhow::Result;
use sysinfo::{ProcessesToUpdate, Signal, System};

use crate::utils::{print_error, print_info, print_success, print_warning};

pub async fn execute(all: bool, port: Option<u16>) -> Result<()> {
    if all {
        stop_all_bridges().await
    } else if let Some(p) = port {
        stop_bridge_on_port(p).await
    } else {
        print_error("Specify either --all or --port <PORT>");
        anyhow::bail!("No target specified");
    }
}

async fn stop_all_bridges() -> Result<()> {
    print_info("Stopping all Icarus bridge instances...");

    let mut system = System::new_all();
    system.refresh_processes(ProcessesToUpdate::All);

    let mut stopped_count = 0;

    for (pid, process) in system.processes() {
        let name = process.name();
        if name == "icarus-bridge" || name == "icarus-bridge.exe" {
            if stop_process(pid.as_u32(), process) {
                stopped_count += 1;
            }
        }
    }

    // Clean up all PID files
    let config_dir = crate::config::IcarusConfig::config_dir()?;
    if config_dir.exists() {
        for entry in std::fs::read_dir(&config_dir)? {
            let entry = entry?;
            let path = entry.path();

            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with("bridge-") && name.ends_with(".pid") {
                    let _ = std::fs::remove_file(&path);
                }
            }
        }
    }

    if stopped_count > 0 {
        print_success(&format!("Stopped {} bridge instance(s)", stopped_count));
    } else {
        print_warning("No bridge instances found");
    }

    Ok(())
}

async fn stop_bridge_on_port(port: u16) -> Result<()> {
    print_info(&format!("Stopping bridge on port {}...", port));

    // First try to find by PID file
    let config_dir = crate::config::IcarusConfig::config_dir()?;
    let pid_file = config_dir.join(format!("bridge-{}.pid", port));

    if pid_file.exists() {
        if let Ok(pid_str) = std::fs::read_to_string(&pid_file) {
            if let Ok(pid) = pid_str.trim().parse::<u32>() {
                let mut system = System::new();
                system.refresh_processes(ProcessesToUpdate::All);

                if let Some(process) = system.process(sysinfo::Pid::from_u32(pid)) {
                    if stop_process(pid, process) {
                        let _ = std::fs::remove_file(&pid_file);
                        print_success(&format!("Stopped bridge on port {} (PID: {})", port, pid));
                        return Ok(());
                    }
                }
            }
        }

        // Clean up stale PID file
        let _ = std::fs::remove_file(&pid_file);
    }

    // If PID file didn't work, search all processes
    let mut system = System::new_all();
    system.refresh_processes(ProcessesToUpdate::All);

    for (pid, process) in system.processes() {
        let name = process.name();
        if name == "icarus-bridge" || name == "icarus-bridge.exe" {
            let cmd_str = process
                .cmd()
                .iter()
                .map(|s| s.to_string_lossy())
                .collect::<Vec<_>>()
                .join(" ");
            if let Some(p) = parse_port_from_cmd(&cmd_str) {
                if p == port {
                    if stop_process(pid.as_u32(), process) {
                        print_success(&format!(
                            "Stopped bridge on port {} (PID: {})",
                            port,
                            pid.as_u32()
                        ));
                        return Ok(());
                    }
                }
            }
        }
    }

    print_warning(&format!("No bridge found on port {}", port));
    Ok(())
}

fn stop_process(pid: u32, process: &sysinfo::Process) -> bool {
    // Try graceful shutdown first
    if process.kill_with(Signal::Term).is_some() {
        // Give it a moment to shut down
        std::thread::sleep(std::time::Duration::from_millis(500));

        // Check if it's still running
        let mut system = System::new();
        system.refresh_processes(ProcessesToUpdate::All);

        if system.process(sysinfo::Pid::from_u32(pid)).is_none() {
            return true;
        }

        // If still running, force kill
        if process.kill_with(Signal::Kill).is_some() {
            return true;
        }
    }

    false
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
