use anyhow::Result;
use std::path::Path;

use crate::utils::{
    build_utils::{build_canister_wasi_native, get_project_name},
    print_info, print_success, print_warning, run_command_interactive,
};

pub async fn execute(force_pure_wasm: bool, skip_optimize: bool) -> Result<()> {
    // Check if we're in an Icarus project
    let current_dir = std::env::current_dir()?;
    if !is_icarus_project(&current_dir) {
        anyhow::bail!("Not in an Icarus project directory. Run this command from a project created with 'icarus new'.");
    }

    // Get project name
    let project_name = get_project_name(&current_dir)?;

    // Build with WASI-Native architecture
    print_info(&format!("Building {}...", project_name));

    // Step 1: Compile the canister
    let target = if force_pure_wasm {
        "wasm32-unknown-unknown"
    } else {
        "wasm32-wasip1"
    };

    run_command_interactive(
        "cargo",
        &["build", "--target", target, "--release"],
        Some(&current_dir),
    )
    .await?;

    // Step 2: Use WASI-Native pipeline for conversion and output
    let final_wasm_path = build_canister_wasi_native(&current_dir, force_pure_wasm).await?;

    // Step 3: Optimize WASM if not skipped
    if !skip_optimize {
        optimize_wasm(&final_wasm_path).await?;
    }

    print_success(&format!(
        "Build completed! WASM available at: {}",
        final_wasm_path.display()
    ));

    Ok(())
}

fn is_icarus_project(path: &Path) -> bool {
    path.join("Cargo.toml").exists() && path.join("dfx.json").exists()
}

async fn optimize_wasm(wasm_path: &Path) -> Result<()> {
    use std::fs;

    // Check if ic-wasm is available
    if which::which("ic-wasm").is_err() {
        print_info(
            "ic-wasm not found. Skipping optimization (install with: cargo install ic-wasm)",
        );
        return Ok(());
    }

    if !wasm_path.exists() {
        print_warning("WASM file not found, skipping optimization");
        return Ok(());
    }

    // Get original size
    let original_size = fs::metadata(wasm_path)?.len();

    print_info("Optimizing WASM with ic-wasm...");

    // Run ic-wasm shrink
    let result = tokio::process::Command::new("ic-wasm")
        .arg(wasm_path)
        .arg("-o")
        .arg(wasm_path)
        .arg("shrink")
        .output()
        .await;

    match result {
        Ok(output) if output.status.success() => {
            // Get new size
            let new_size = fs::metadata(wasm_path)?.len();
            let reduction = ((original_size - new_size) as f64 / original_size as f64) * 100.0;

            print_success(&format!(
                "WASM optimized: {} → {} (reduced by {:.1}%)",
                format_size(original_size),
                format_size(new_size),
                reduction
            ));
        }
        Ok(output) => {
            let error = String::from_utf8_lossy(&output.stderr);
            print_warning(&format!("WASM optimization failed: {}", error));
        }
        Err(e) => {
            print_warning(&format!("Could not run ic-wasm: {}", e));
        }
    }

    Ok(())
}

fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}
