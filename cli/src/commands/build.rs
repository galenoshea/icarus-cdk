use anyhow::Result;
use std::io::Write;
use std::path::Path;
use std::process::Command;

use crate::utils::{create_progress_bar, print_error, print_info, print_success, print_warning};

pub async fn execute(
    no_optimize: bool,
    optimize_size: bool,
    optimize_performance: bool,
    compress: bool,
    output_dir: Option<String>,
) -> Result<()> {
    // Check if we're in an Icarus project
    let current_dir = std::env::current_dir()
        .map_err(|e| anyhow::anyhow!("Failed to get current directory: {}", e))?;

    if !is_icarus_project(&current_dir) {
        anyhow::bail!("Not in an Icarus project directory. Run this command from a project created with 'icarus new'.");
    }

    // Show build profile if applicable
    let profile_msg = if no_optimize {
        "Building with debug profile (no optimizations)...".to_string()
    } else if optimize_size {
        "Building with size profile (maximum compression)...".to_string()
    } else if optimize_performance {
        "Building with speed profile (maximum performance)...".to_string()
    } else {
        "Building with default profile (balanced optimization)...".to_string()
    };
    print_info(&profile_msg);

    // Step 1: Build with cargo (always release mode for optimal size)
    let cargo_args = vec!["build", "--target", "wasm32-unknown-unknown", "--release"];

    let pb = create_progress_bar(100, "Compiling Rust code");

    let output = Command::new("cargo")
        .args(&cargo_args)
        .current_dir(&current_dir)
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                anyhow::anyhow!(
                    "cargo not found. Please ensure Rust is installed and cargo is in your PATH"
                )
            } else {
                anyhow::anyhow!("Failed to run cargo: {}", e)
            }
        })?;

    if !output.status.success() {
        pb.finish_and_clear();
        print_error("Cargo build failed");
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        anyhow::bail!("Build failed");
    }

    pb.set_position(50);
    pb.set_message("Build complete".to_string());

    // Step 2: Optimize WASM (unless disabled)
    if !no_optimize {
        pb.set_message("Optimizing WASM".to_string());

        let target_dir = "release";
        let wasm_path = current_dir
            .join("target")
            .join("wasm32-unknown-unknown")
            .join(target_dir);

        // Find the WASM file
        let wasm_file = find_wasm_file(&wasm_path)?;

        // Note: ic-wasm integration temporarily removed due to compatibility issues
        // TODO: Re-add ic-wasm support once the execution issue is resolved

        // Then optimize with wasm-opt
        if let Ok(wasm_opt_binary) = which::which("wasm-opt") {
            pb.set_message("Optimizing WASM with wasm-opt".to_string());
            let optimized_path = wasm_file.with_extension("opt.wasm");

            // Determine optimization level based on flags
            let opt_level = if optimize_size {
                "-Oz" // Maximum size reduction
            } else if optimize_performance {
                "-O4" // Maximum performance
            } else {
                "-O3" // Balanced (default for cycles)
            };

            let mut args = vec![
                opt_level,
                wasm_file.to_str().unwrap(),
                "-o",
                optimized_path.to_str().unwrap(),
            ];

            // Add custom settings for O4
            if optimize_performance {
                args.push("--flexible-inline-max-function-size");
                args.push("20000");
            }

            let output = Command::new(&wasm_opt_binary)
                .args(&args)
                .output()
                .map_err(|e| anyhow::anyhow!("Failed to run wasm-opt: {}", e))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                print_warning(&format!("wasm-opt warning: {}", stderr));
            }

            // Replace original with optimized
            std::fs::rename(&optimized_path, &wasm_file)?;

            pb.set_position(75);
            pb.set_message("WASM optimized".to_string());
        } else {
            print_info("wasm-opt not found. Consider installing for better optimization.");
        }
    }

    // Step 3: Extract Candid interface from WASM
    pb.set_message("Extracting Candid interface".to_string());
    extract_candid_from_wasm(&current_dir)
        .map_err(|e| anyhow::anyhow!("Failed to extract Candid interface: {}", e))?;
    pb.set_position(85);

    // Step 4: Create .dfx directories if needed (for deployment)
    pb.set_message("Creating build directories".to_string());
    create_dfx_directories(&current_dir)
        .map_err(|e| anyhow::anyhow!("Failed to create build directories: {}", e))?;

    // Step 5: Finalize build
    pb.set_message("Finalizing build".to_string());

    // Step 5: Gzip compression (enabled by default)
    if compress {
        pb.set_message("Compressing WASM with gzip".to_string());
        compress_wasm(&current_dir)
            .map_err(|e| anyhow::anyhow!("Failed to compress WASM: {}", e))?;
        pb.set_position(95);
    } else {
        print_info("Skipping gzip compression (use --no-compress flag)");
    }

    pb.set_position(100);
    pb.finish_and_clear();

    print_success("Build completed successfully!");

    // Copy artifacts to output directory if specified
    if let Some(output) = output_dir {
        copy_artifacts_to_output(&current_dir, &output)?;
    }

    // Show build artifacts info
    print_build_summary(&current_dir, compress)?;

    Ok(())
}

fn is_icarus_project(path: &Path) -> bool {
    path.join("Cargo.toml").exists() && path.join("dfx.json").exists()
}

fn copy_artifacts_to_output(project_dir: &Path, output_dir: &str) -> Result<()> {
    let output_path = project_dir.join(output_dir);

    // Create output directory if it doesn't exist
    std::fs::create_dir_all(&output_path)?;

    let project_name = get_project_name(project_dir)?;
    let wasm_source_dir = project_dir
        .join("target")
        .join("wasm32-unknown-unknown")
        .join("release");

    // Copy WASM file
    if let Ok(wasm_file) = find_wasm_file(&wasm_source_dir) {
        let dest_wasm = output_path.join(wasm_file.file_name().unwrap());
        std::fs::copy(&wasm_file, &dest_wasm)?;

        // Copy compressed WASM if it exists
        let gz_file = wasm_file.with_extension("wasm.gz");
        if gz_file.exists() {
            let dest_gz = output_path.join(gz_file.file_name().unwrap());
            std::fs::copy(&gz_file, &dest_gz)?;
        }
    }

    // Copy Candid file if it exists
    let candid_file = wasm_source_dir.join(format!("{}.did", project_name));
    if candid_file.exists() {
        let dest_candid = output_path.join(candid_file.file_name().unwrap());
        std::fs::copy(&candid_file, &dest_candid)?;
    }

    print_success(&format!("Artifacts copied to {}/", output_dir));
    Ok(())
}

fn create_dfx_directories(project_dir: &Path) -> Result<()> {
    let project_name = get_project_name(project_dir)?;

    // Create .dfx/local/canisters/<project> directory structure
    let canister_dir = project_dir
        .join(".dfx")
        .join("local")
        .join("canisters")
        .join(&project_name);

    if !canister_dir.exists() {
        std::fs::create_dir_all(&canister_dir)?;
    }

    Ok(())
}

fn find_wasm_file(dir: &Path) -> Result<std::path::PathBuf> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("wasm") {
            return Ok(path);
        }
    }
    anyhow::bail!("No WASM file found in target directory")
}

fn compress_wasm(project_dir: &Path) -> Result<()> {
    let project_name = get_project_name(project_dir)?;
    let wasm_path = project_dir
        .join("target")
        .join("wasm32-unknown-unknown")
        .join("release")
        .join(format!("{}.wasm", project_name.replace('-', "_")));

    if !wasm_path.exists() {
        anyhow::bail!("WASM file not found for compression");
    }

    // Read the WASM file
    let wasm_data = std::fs::read(&wasm_path)?;

    // Create gzipped version
    let gz_path = wasm_path.with_extension("wasm.gz");
    let gz_file = std::fs::File::create(&gz_path)?;
    let mut encoder = flate2::write::GzEncoder::new(gz_file, flate2::Compression::best());
    encoder.write_all(&wasm_data)?;
    encoder.finish()?;

    Ok(())
}

fn extract_candid_from_wasm(project_dir: &Path) -> Result<()> {
    // Check if candid-extractor is installed
    if which::which("candid-extractor").is_err() {
        print_info("candid-extractor not found. Installing...");

        // Install candid-extractor
        let output = Command::new("cargo")
            .args(&["install", "candid-extractor"])
            .output()
            .map_err(|e| anyhow::anyhow!("Failed to run cargo install: {}", e))?;

        if !output.status.success() {
            print_warning("Failed to install candid-extractor. Candid file will be generated during deployment.");
            return Ok(());
        }
    }

    let project_name = get_project_name(project_dir)?;
    let target_dir = "release";
    let wasm_path = project_dir
        .join("target")
        .join("wasm32-unknown-unknown")
        .join(target_dir)
        .join(format!("{}.wasm", project_name.replace('-', "_")));

    if !wasm_path.exists() {
        print_warning("WASM file not found. Candid file will be generated during deployment.");
        return Ok(());
    }

    // Extract Candid using candid-extractor
    let output = Command::new("candid-extractor")
        .arg(&wasm_path)
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to run candid-extractor: {}", e))?;

    if output.status.success() {
        let candid_content = String::from_utf8_lossy(&output.stdout);

        // Save to src/<project>.did
        let candid_path = project_dir
            .join("src")
            .join(format!("{}.did", project_name));
        std::fs::write(&candid_path, candid_content.as_ref())?;

        // Also update .dfx directory if it exists
        let dfx_candid_path = project_dir
            .join(".dfx")
            .join("local")
            .join("canisters")
            .join(&project_name)
            .join("service.did");

        if let Some(parent) = dfx_candid_path.parent() {
            if parent.exists() {
                std::fs::write(&dfx_candid_path, candid_content.as_ref())?;
            }
        }
    } else {
        print_warning(
            "Failed to extract Candid interface. It will be generated during deployment.",
        );
    }

    Ok(())
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

fn print_build_summary(project_dir: &Path, show_compressed: bool) -> Result<()> {
    let project_name = get_project_name(project_dir)?;
    let wasm_path = project_dir
        .join("target")
        .join("wasm32-unknown-unknown")
        .join("release");

    if let Ok(wasm_file) = find_wasm_file(&wasm_path) {
        let metadata = std::fs::metadata(&wasm_file)?;
        let size_kb = metadata.len() as f64 / 1_024.0;

        if show_compressed {
            let gz_path = wasm_file.with_extension("wasm.gz");
            if gz_path.exists() {
                let gz_metadata = std::fs::metadata(&gz_path)?;
                let gz_size_kb = gz_metadata.len() as f64 / 1_024.0;
                let compression_ratio =
                    100.0 - (gz_metadata.len() as f64 / metadata.len() as f64 * 100.0);

                println!(
                    "  📦 WASM: {:.1}KB → {:.1}KB compressed ({:.0}% reduction)",
                    size_kb, gz_size_kb, compression_ratio
                );
            } else {
                println!("  📦 WASM: {:.1}KB", size_kb);
            }
        } else {
            println!("  📦 WASM: {:.1}KB", size_kb);
        }

        // Check for Candid metadata
        check_candid_metadata(&wasm_file, &project_name, project_dir);

        // Only show wasm-opt hint if not installed
        if which::which("wasm-opt").is_err() {
            println!("  💡 Tip: Install wasm-opt for better optimization: npm install -g wasm-opt");
        }
    }

    Ok(())
}

fn check_candid_metadata(wasm_file: &Path, project_name: &str, project_dir: &Path) {
    // Check if ic-wasm is available
    if let Ok(ic_wasm) = which::which("ic-wasm") {
        // Check for candid:service metadata
        let output = Command::new(ic_wasm)
            .arg(wasm_file)
            .arg("metadata")
            .arg("candid:service")
            .output()
            .ok();

        let has_candid = output.map(|o| o.status.success()).unwrap_or(false);

        if has_candid {
            println!("  ✅ Candid metadata embedded (Candid UI will work)");
        } else {
            // Check if .dfx version exists with Candid
            let dfx_wasm = project_dir
                .join(".dfx")
                .join("local")
                .join("canisters")
                .join(project_name)
                .join(format!("{}.wasm", project_name));

            if dfx_wasm.exists() {
                println!("  ⚠️  No Candid in target/ WASM. Use .dfx/local/canisters/{}/{}.wasm for deployment", project_name, project_name);
            } else {
                println!("  ⚠️  No Candid metadata. Run 'dfx build' to embed Candid interface");
            }
        }
    }
}
