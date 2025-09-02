use anyhow::Result;
use colored::Colorize;
use std::path::Path;
use std::process::Command;

use crate::utils::print_error;

pub async fn execute(top: usize, check_compressed: bool) -> Result<()> {
    // Check if we're in an Icarus project
    let current_dir = std::env::current_dir()?;
    if !is_icarus_project(&current_dir) {
        anyhow::bail!("Not in an Icarus project directory");
    }

    // Get project name and paths
    let project_name = get_project_name(&current_dir)?;
    let wasm_path = current_dir
        .join("target")
        .join("wasm32-unknown-unknown")
        .join("release")
        .join(format!("{}.wasm", project_name.replace('-', "_")));

    if !wasm_path.exists() {
        anyhow::bail!("WASM file not found. Run 'icarus build' first.");
    }

    // Show basic size info
    println!("\n{}", "📊 WASM Binary Analysis".bold().cyan());
    println!(
        "{}",
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan()
    );

    let metadata = std::fs::metadata(&wasm_path)?;
    let size_mb = metadata.len() as f64 / 1_048_576.0;
    let size_kb = metadata.len() as f64 / 1_024.0;

    println!("\n{}", "File sizes:".bold());
    if size_mb < 1.0 {
        println!("  Original WASM: {:.2} KB", size_kb);
    } else {
        println!("  Original WASM: {:.2} MB", size_mb);
    }

    // Check compressed size
    let gz_path = wasm_path.with_extension("wasm.gz");
    if gz_path.exists() {
        let gz_metadata = std::fs::metadata(&gz_path)?;
        let gz_size_kb = gz_metadata.len() as f64 / 1_024.0;
        let compression_ratio = 100.0 - (gz_metadata.len() as f64 / metadata.len() as f64 * 100.0);

        println!(
            "  Compressed:    {:.2} KB ({:.1}% reduction)",
            gz_size_kb, compression_ratio
        );
    } else if check_compressed {
        println!(
            "  Compressed:    {} (run 'icarus build' to generate)",
            "not found".yellow()
        );
    }

    // Check if twiggy is installed
    if which::which("twiggy").is_err() {
        println!("\n{}", "⚠️  Twiggy not installed".yellow());
        println!(
            "Install it for detailed analysis: {}",
            "cargo install twiggy".bright_blue()
        );

        // Still show what we can
        show_basic_analysis(&wasm_path)?;
    } else {
        // Run twiggy analysis
        println!("\n{}", "Top size contributors:".bold());
        println!("{}", "─".repeat(50));

        let output = Command::new("twiggy")
            .args(&["top", "-n", &top.to_string(), wasm_path.to_str().unwrap()])
            .output()?;

        if output.status.success() {
            println!("{}", String::from_utf8_lossy(&output.stdout));
        } else {
            print_error("Failed to run twiggy analysis");
        }

        // Show optimization suggestions
        show_optimization_suggestions(&wasm_path)?;
    }

    Ok(())
}

fn show_basic_analysis(wasm_path: &Path) -> Result<()> {
    println!("\n{}", "Basic analysis:".bold());

    // Check for common optimization opportunities
    let wasm_data = std::fs::read(wasm_path)?;
    let wasm_size = wasm_data.len();

    // Rough heuristics
    if wasm_size > 2_000_000 {
        println!("  ⚠️  WASM size over 2MB - optimization highly recommended");
        println!("     Consider using 'icarus build --profile=size'");
    } else if wasm_size > 1_000_000 {
        println!("  ⚠️  WASM size over 1MB - optimization recommended");
    } else if wasm_size < 500_000 {
        println!("  ✅ WASM size under 500KB - well optimized");
    }

    Ok(())
}

fn show_optimization_suggestions(wasm_path: &Path) -> Result<()> {
    println!("\n{}", "💡 Optimization suggestions:".bold().yellow());

    let wasm_size = std::fs::metadata(wasm_path)?.len();

    if wasm_size > 1_000_000 {
        println!(
            "  1. Use size profile: {}",
            "icarus build --profile=size".bright_blue()
        );
        println!("  2. Remove unused dependencies from Cargo.toml");
        println!("  3. Use default features = false for large crates");
        println!("  4. Consider lighter alternatives (e.g., ciborium vs serde_cbor)");
    }

    let gz_path = wasm_path.with_extension("wasm.gz");
    if !gz_path.exists() {
        println!("  • Enable compression: {}", "icarus build".bright_blue());
        println!("    (Compression is now enabled by default)");
    }

    // Check optimization tools
    let ic_wasm_installed = which::which("ic-wasm").is_ok();
    let wasm_opt_installed = which::which("wasm-opt").is_ok();

    if !ic_wasm_installed || !wasm_opt_installed {
        println!("\n{}", "Missing optimization tools:".bold());
        if !ic_wasm_installed {
            println!(
                "  • Install ic-wasm: {}",
                "cargo install ic-wasm".bright_blue()
            );
        }
        if !wasm_opt_installed {
            println!(
                "  • Install wasm-opt: {}",
                "npm install -g wasm-opt".bright_blue()
            );
        }
    }

    Ok(())
}

fn is_icarus_project(path: &Path) -> bool {
    path.join("Cargo.toml").exists() && path.join("dfx.json").exists()
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
