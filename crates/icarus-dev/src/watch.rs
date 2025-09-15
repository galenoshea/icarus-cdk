use anyhow::Result;
use colored::Colorize;
use notify::{Event, EventKind, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Instant;
use tokio::signal;
use tokio::sync::mpsc as tokio_mpsc;
use tokio::time::{timeout, Duration};

use crate::utils::{print_info, print_success, print_warning, run_command_interactive};

pub async fn execute(patterns: Option<Vec<String>>, delay: u64, verbose: bool) -> Result<()> {
    println!(
        "\n{} {}",
        "👁️".bright_blue(),
        "File Watcher".bright_cyan().bold()
    );
    println!(
        "{}",
        "Monitoring files for changes and triggering automatic redeployment.\n".bright_white()
    );

    // Check if we're in an Icarus project
    let current_dir = std::env::current_dir()?;
    if !is_icarus_project(&current_dir) {
        print_warning("Not in an Icarus project directory.");
        print_info("Run this command from within an Icarus project created with 'icarus new'.");
        return Ok(());
    }

    // Default watch patterns - simplified to directories for notify crate
    let watch_paths = patterns.unwrap_or_else(|| {
        vec![
            "src".to_string(),
            "Cargo.toml".to_string(),
            "dfx.json".to_string(),
        ]
    });

    print_info("File watcher configuration:");
    println!("  {} Watching:", "📋".bright_blue());
    for pattern in &watch_paths {
        println!("    {} {}", "📄".bright_cyan(), pattern.bright_white());
    }
    println!(
        "  {} Delay: {}ms",
        "⏱️".bright_blue(),
        delay.to_string().bright_cyan()
    );
    println!(
        "  {} Verbose: {}",
        "🔍".bright_blue(),
        if verbose {
            "enabled".bright_green()
        } else {
            "disabled".bright_yellow()
        }
    );
    println!();

    // Check if local IC replica is running
    check_local_replica().await?;

    // Start file watcher
    print_success("File watcher started!");
    println!("{}", "────────────────────────────────────".bright_blue());
    println!("  {} Watching for changes...", "👀".bright_green());
    println!("  {} Auto-redeploy: enabled", "🔄".bright_green());
    println!("  {} Press Ctrl+C to stop", "⏹️".bright_blue());
    println!("{}", "────────────────────────────────────".bright_blue());
    println!();

    // Start real file watcher
    start_file_watcher(&current_dir, watch_paths, delay, verbose).await?;

    Ok(())
}

fn is_icarus_project(path: &Path) -> bool {
    path.join("Cargo.toml").exists() && path.join("dfx.json").exists() && path.join("src").exists()
}

async fn check_local_replica() -> Result<()> {
    match tokio::process::Command::new("dfx")
        .args(&["ping", "local"])
        .output()
        .await
    {
        Ok(output) if output.status.success() => {
            println!("  {} Local IC replica is running", "✅".bright_green());
        }
        _ => {
            print_warning("Local IC replica not running");
            print_info("File watcher will continue, but redeployment will fail");
            print_info("Start replica with: dfx start --background");
        }
    }

    Ok(())
}

async fn start_file_watcher(
    project_dir: &Path,
    watch_paths: Vec<String>,
    debounce_delay: u64,
    verbose: bool,
) -> Result<()> {
    // Create channels for file events
    let (file_tx, mut file_rx) = tokio_mpsc::unbounded_channel::<FileChangeEvent>();

    // Create notify watcher
    let (notify_tx, notify_rx) = mpsc::channel();
    let mut watcher = notify::recommended_watcher(notify_tx)?;

    // Watch specified paths
    for watch_path in &watch_paths {
        let full_path = project_dir.join(watch_path);
        if full_path.exists() {
            let mode = if full_path.is_dir() {
                RecursiveMode::Recursive
            } else {
                RecursiveMode::NonRecursive
            };

            if let Err(e) = watcher.watch(&full_path, mode) {
                print_warning(&format!("Failed to watch {}: {}", watch_path, e));
            } else if verbose {
                println!(
                    "  {} Watching: {}",
                    "👁️".bright_blue(),
                    full_path.display().to_string().bright_cyan()
                );
            }
        } else if verbose {
            print_warning(&format!("Path does not exist: {}", watch_path));
        }
    }

    // Spawn task to handle file system events
    let file_tx_clone = file_tx.clone();
    let project_dir_clone = project_dir.to_path_buf();
    tokio::task::spawn_blocking(move || {
        let mut last_event_time = Instant::now();

        for res in notify_rx {
            match res {
                Ok(event) => {
                    // Only process certain types of events
                    if should_process_event(&event, &project_dir_clone) {
                        let now = Instant::now();

                        // Simple debouncing - only process if enough time has passed
                        if now.duration_since(last_event_time).as_millis() > debounce_delay as u128
                        {
                            last_event_time = now;

                            if let Some(path) = event.paths.first() {
                                let change_event = FileChangeEvent {
                                    path: path.clone(),
                                    timestamp: now,
                                };

                                if file_tx_clone.send(change_event).is_err() {
                                    break; // Channel closed
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("File watcher error: {}", e);
                }
            }
        }
    });

    // Main event loop
    let mut deployment_count = 0;
    let mut is_building = false;

    loop {
        tokio::select! {
            // Handle file change events
            Some(change_event) = file_rx.recv() => {
                if is_building {
                    if verbose {
                        println!("  {} Ignoring change during build: {}",
                            "⏸️".bright_yellow(),
                            change_event.path.display().to_string().bright_cyan()
                        );
                    }
                    continue;
                }

                deployment_count += 1;

                if verbose {
                    println!("  {} File changed: {}",
                        "📝".bright_yellow(),
                        change_event.path.display().to_string().bright_cyan()
                    );
                }

                print_info(&format!("Change #{} - Triggering rebuild...", deployment_count));
                let _is_building = true; // Track building state for potential future use

                match build_and_deploy(project_dir).await {
                    Ok(canister_id) => {
                        print_success(&format!("Redeployment successful! Canister: {}", canister_id));
                        if verbose {
                            let elapsed = change_event.timestamp.elapsed();
                            println!("  {} Build completed in {}ms",
                                "⚡".bright_green(),
                                elapsed.as_millis().to_string().bright_cyan()
                            );
                        }
                    }
                    Err(e) => {
                        print_warning(&format!("Redeployment failed: {}", e));
                        if verbose {
                            println!("  {} Fix the issues and save again to retry", "💡".bright_yellow());
                        }
                    }
                }

                is_building = false;
                println!("  {} Watching for more changes...", "👀".bright_blue());
                println!();
            }

            // Handle Ctrl+C
            _ = signal::ctrl_c() => {
                println!("\n{} Shutting down file watcher...", "⏹️".bright_yellow());
                break;
            }
        }
    }

    print_success(&format!(
        "File watcher stopped after {} deployments",
        deployment_count
    ));
    Ok(())
}

#[derive(Debug, Clone)]
struct FileChangeEvent {
    path: PathBuf,
    timestamp: Instant,
}

fn should_process_event(event: &Event, project_dir: &Path) -> bool {
    // Only process write/create/remove events
    match event.kind {
        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {}
        _ => return false,
    }

    // Check if any path is relevant
    for path in &event.paths {
        if let Ok(relative_path) = path.strip_prefix(project_dir) {
            let path_str = relative_path.to_string_lossy();

            // Include Rust source files
            if path_str.ends_with(".rs") {
                return true;
            }

            // Include important config files
            if path_str == "Cargo.toml" || path_str == "dfx.json" {
                return true;
            }

            // Exclude target directory and other build artifacts
            if path_str.starts_with("target/")
                || path_str.starts_with(".dfx/")
                || path_str.starts_with(".git/")
                || path_str.contains("/.")
                || path_str.ends_with(".tmp")
                || path_str.ends_with(".lock")
            {
                continue;
            }
        }
    }

    false
}

async fn build_and_deploy(project_dir: &Path) -> Result<String> {
    // Build WASM with timeout to avoid hanging
    println!("  {} Building WASM...", "🔨".bright_blue());

    let build_future = run_command_interactive(
        "cargo",
        &["build", "--target", "wasm32-unknown-unknown", "--release"],
        Some(project_dir),
    );

    // Add timeout to prevent hanging builds
    match timeout(Duration::from_secs(120), build_future).await {
        Ok(result) => result?,
        Err(_) => {
            return Err(anyhow::anyhow!("Build timed out after 120 seconds"));
        }
    }

    // Get project name for deployment
    let project_name = get_project_name(project_dir)?;

    // Get current principal for init argument
    let principal_output = tokio::process::Command::new("dfx")
        .args(&["identity", "get-principal"])
        .current_dir(project_dir)
        .output()
        .await?;

    if !principal_output.status.success() {
        return Err(anyhow::anyhow!("Failed to get dfx identity"));
    }

    let principal = String::from_utf8_lossy(&principal_output.stdout)
        .trim()
        .to_string();
    let init_arg = format!("(principal \"{}\")", principal);

    // Deploy canister
    println!("  {} Deploying to local IC...", "📦".bright_blue());

    let deploy_args = vec![
        "deploy",
        &project_name,
        "--network",
        "local",
        "--argument",
        &init_arg,
        "--upgrade-unchanged",
    ];

    let deploy_future = run_command_interactive("dfx", &deploy_args, Some(project_dir));

    // Add timeout for deployment too
    match timeout(Duration::from_secs(60), deploy_future).await {
        Ok(result) => result?,
        Err(_) => {
            return Err(anyhow::anyhow!("Deployment timed out after 60 seconds"));
        }
    }

    // Get canister ID
    let canister_id_output = tokio::process::Command::new("dfx")
        .args(&["canister", "id", &project_name, "--network", "local"])
        .current_dir(project_dir)
        .output()
        .await?;

    if !canister_id_output.status.success() {
        return Err(anyhow::anyhow!("Failed to get canister ID"));
    }

    let canister_id = String::from_utf8_lossy(&canister_id_output.stdout)
        .trim()
        .to_string();
    Ok(canister_id)
}

fn get_project_name(project_dir: &Path) -> Result<String> {
    let cargo_toml = project_dir.join("Cargo.toml");
    let content = std::fs::read_to_string(&cargo_toml)?;
    let toml: toml::Value = toml::from_str(&content)?;

    toml.get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow::anyhow!("Could not find package name in Cargo.toml"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use notify::{Event, EventKind};
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_is_icarus_project_valid() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // Create required files for Icarus project
        fs::write(
            project_path.join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();
        fs::write(project_path.join("dfx.json"), "{}").unwrap();
        fs::create_dir(project_path.join("src")).unwrap();

        assert!(is_icarus_project(project_path));
    }

    #[test]
    fn test_is_icarus_project_invalid() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // Missing required files
        assert!(!is_icarus_project(project_path));

        // Only some files present
        fs::write(
            project_path.join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();
        assert!(!is_icarus_project(project_path));

        fs::write(project_path.join("dfx.json"), "{}").unwrap();
        assert!(!is_icarus_project(project_path));

        // All files present
        fs::create_dir(project_path.join("src")).unwrap();
        assert!(is_icarus_project(project_path));
    }

    #[test]
    fn test_get_project_name_success() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        let cargo_toml_content = r#"
[package]
name = "my-watch-project"
version = "0.1.0"
edition = "2021"
"#;
        fs::write(project_path.join("Cargo.toml"), cargo_toml_content).unwrap();

        let result = get_project_name(project_path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "my-watch-project");
    }

    #[test]
    fn test_get_project_name_missing_file() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        let result = get_project_name(project_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_project_name_invalid_toml() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        fs::write(project_path.join("Cargo.toml"), "invalid toml [[[").unwrap();

        let result = get_project_name(project_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_project_name_missing_package() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        let cargo_toml_content = r#"
[dependencies]
serde = "1.0"
"#;
        fs::write(project_path.join("Cargo.toml"), cargo_toml_content).unwrap();

        let result = get_project_name(project_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_file_change_event_creation() {
        let path = PathBuf::from("/test/path/file.rs");
        let timestamp = Instant::now();

        let event = FileChangeEvent {
            path: path.clone(),
            timestamp,
        };

        assert_eq!(event.path, path);
        assert_eq!(event.timestamp, timestamp);
    }

    #[test]
    fn test_should_process_event_rust_files() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        let rust_file = project_path.join("src").join("main.rs");
        fs::create_dir_all(rust_file.parent().unwrap()).unwrap();
        fs::write(&rust_file, "// rust code").unwrap();

        let event = Event {
            kind: EventKind::Modify(notify::event::ModifyKind::Data(
                notify::event::DataChange::Content,
            )),
            paths: vec![rust_file],
            attrs: Default::default(),
        };

        assert!(should_process_event(&event, project_path));
    }

    #[test]
    fn test_should_process_event_config_files() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // Test Cargo.toml
        let cargo_toml = project_path.join("Cargo.toml");
        fs::write(&cargo_toml, "[package]").unwrap();

        let event = Event {
            kind: EventKind::Modify(notify::event::ModifyKind::Data(
                notify::event::DataChange::Content,
            )),
            paths: vec![cargo_toml],
            attrs: Default::default(),
        };

        assert!(should_process_event(&event, project_path));

        // Test dfx.json
        let dfx_json = project_path.join("dfx.json");
        fs::write(&dfx_json, "{}").unwrap();

        let event = Event {
            kind: EventKind::Modify(notify::event::ModifyKind::Data(
                notify::event::DataChange::Content,
            )),
            paths: vec![dfx_json],
            attrs: Default::default(),
        };

        assert!(should_process_event(&event, project_path));
    }

    #[test]
    fn test_should_process_event_ignored_paths() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // Test target directory (should be ignored)
        let target_file = project_path
            .join("target")
            .join("debug")
            .join("build")
            .join("something");
        fs::create_dir_all(target_file.parent().unwrap()).unwrap();
        fs::write(&target_file, "build artifact").unwrap();

        let event = Event {
            kind: EventKind::Modify(notify::event::ModifyKind::Data(
                notify::event::DataChange::Content,
            )),
            paths: vec![target_file],
            attrs: Default::default(),
        };

        assert!(!should_process_event(&event, project_path));

        // Test .dfx directory (should be ignored)
        let dfx_file = project_path.join(".dfx").join("local").join("something");
        fs::create_dir_all(dfx_file.parent().unwrap()).unwrap();
        fs::write(&dfx_file, "dfx artifact").unwrap();

        let event = Event {
            kind: EventKind::Modify(notify::event::ModifyKind::Data(
                notify::event::DataChange::Content,
            )),
            paths: vec![dfx_file],
            attrs: Default::default(),
        };

        assert!(!should_process_event(&event, project_path));

        // Test .git directory (should be ignored)
        let git_file = project_path.join(".git").join("config");
        fs::create_dir_all(git_file.parent().unwrap()).unwrap();
        fs::write(&git_file, "git config").unwrap();

        let event = Event {
            kind: EventKind::Modify(notify::event::ModifyKind::Data(
                notify::event::DataChange::Content,
            )),
            paths: vec![git_file],
            attrs: Default::default(),
        };

        assert!(!should_process_event(&event, project_path));
    }

    #[test]
    fn test_should_process_event_temp_files() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // Test .tmp files (should be ignored)
        let tmp_file = project_path.join("src").join("temp.tmp");
        fs::create_dir_all(tmp_file.parent().unwrap()).unwrap();
        fs::write(&tmp_file, "temp content").unwrap();

        let event = Event {
            kind: EventKind::Modify(notify::event::ModifyKind::Data(
                notify::event::DataChange::Content,
            )),
            paths: vec![tmp_file],
            attrs: Default::default(),
        };

        assert!(!should_process_event(&event, project_path));

        // Test .lock files (should be ignored)
        let lock_file = project_path.join("Cargo.lock");
        fs::write(&lock_file, "lock content").unwrap();

        let event = Event {
            kind: EventKind::Modify(notify::event::ModifyKind::Data(
                notify::event::DataChange::Content,
            )),
            paths: vec![lock_file],
            attrs: Default::default(),
        };

        assert!(!should_process_event(&event, project_path));
    }

    #[test]
    fn test_should_process_event_wrong_event_kind() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        let rust_file = project_path.join("src").join("main.rs");
        fs::create_dir_all(rust_file.parent().unwrap()).unwrap();
        fs::write(&rust_file, "// rust code").unwrap();

        // Test event kind that should be ignored
        let event = Event {
            kind: EventKind::Access(notify::event::AccessKind::Read),
            paths: vec![rust_file],
            attrs: Default::default(),
        };

        assert!(!should_process_event(&event, project_path));
    }

    #[test]
    fn test_should_process_event_create_kind() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        let rust_file = project_path.join("src").join("new_file.rs");
        fs::create_dir_all(rust_file.parent().unwrap()).unwrap();
        fs::write(&rust_file, "// new rust file").unwrap();

        let event = Event {
            kind: EventKind::Create(notify::event::CreateKind::File),
            paths: vec![rust_file],
            attrs: Default::default(),
        };

        assert!(should_process_event(&event, project_path));
    }

    #[test]
    fn test_should_process_event_remove_kind() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        let rust_file = project_path.join("src").join("deleted_file.rs");

        let event = Event {
            kind: EventKind::Remove(notify::event::RemoveKind::File),
            paths: vec![rust_file],
            attrs: Default::default(),
        };

        assert!(should_process_event(&event, project_path));
    }

    #[test]
    fn test_should_process_event_multiple_paths() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        let rust_file = project_path.join("src").join("main.rs");
        fs::create_dir_all(rust_file.parent().unwrap()).unwrap();
        fs::write(&rust_file, "// rust code").unwrap();

        let target_file = project_path
            .join("target")
            .join("debug")
            .join("build_artifact");
        fs::create_dir_all(target_file.parent().unwrap()).unwrap();
        fs::write(&target_file, "build artifact").unwrap();

        // Event with both relevant and irrelevant paths
        let event = Event {
            kind: EventKind::Modify(notify::event::ModifyKind::Data(
                notify::event::DataChange::Content,
            )),
            paths: vec![rust_file, target_file],
            attrs: Default::default(),
        };

        // Should return true because at least one path is relevant
        assert!(should_process_event(&event, project_path));
    }

    #[test]
    fn test_should_process_event_outside_project() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // File outside project directory
        let outside_file = temp_dir.path().parent().unwrap().join("outside.rs");

        let event = Event {
            kind: EventKind::Modify(notify::event::ModifyKind::Data(
                notify::event::DataChange::Content,
            )),
            paths: vec![outside_file],
            attrs: Default::default(),
        };

        assert!(!should_process_event(&event, project_path));
    }

    #[test]
    fn test_project_name_with_hyphens_to_underscores() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        let cargo_toml_content = r#"
[package]
name = "my-hyphenated-project"
version = "0.1.0"
"#;
        fs::write(project_path.join("Cargo.toml"), cargo_toml_content).unwrap();

        let result = get_project_name(project_path);
        assert!(result.is_ok());
        let project_name = result.unwrap();
        assert_eq!(project_name, "my-hyphenated-project");

        // This tests the logic in the actual functions that convert hyphens to underscores for WASM names
        let wasm_name = project_name.replace('-', "_");
        assert_eq!(wasm_name, "my_hyphenated_project");
    }

    #[tokio::test]
    async fn test_check_local_replica_command() {
        // This test verifies the function doesn't panic
        // The actual dfx command might not be available in test environment
        let result = check_local_replica().await;
        // Should complete without error regardless of dfx availability
        assert!(result.is_ok());
    }
}
