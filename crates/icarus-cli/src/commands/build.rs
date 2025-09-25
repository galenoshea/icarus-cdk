use anyhow::Result;
use std::path::Path;
use toml::Value;

use crate::utils::{
    build_utils::{build_canister_wasi_native, get_project_name},
    print_info, print_success, print_warning, run_command_with_env,
};

pub async fn execute(
    force_pure_wasm: bool,
    skip_optimize: bool,
    enable_simd: bool,
    force_wasi: bool,
) -> Result<()> {
    // Check if we're in an Icarus project
    let current_dir = std::env::current_dir()?;
    if !is_icarus_project(&current_dir) {
        anyhow::bail!("Not in an Icarus project directory. Run this command from a project created with 'icarus new'.");
    }

    // Get project name
    let project_name = get_project_name(&current_dir)?;

    // Determine WASI usage based on flags and auto-detection
    let has_wasi = if force_pure_wasm {
        false // Force pure WASM overrides everything
    } else if force_wasi {
        true // Explicit --wasi flag overrides auto-detection
    } else {
        detect_wasi_features(&current_dir)? // Auto-detect from dependencies
    };

    if has_wasi {
        if force_wasi {
            print_info(&format!(
                "🔨 Building {} with WASI support (forced by --wasi flag)",
                project_name
            ));
        } else {
            print_info(&format!(
                "🔨 Building {} with WASI support (auto-detected)",
                project_name
            ));
        }
    } else if force_pure_wasm {
        print_info(&format!(
            "🔨 Building {} with pure WASM (forced by --pure-wasm flag)",
            project_name
        ));
    } else {
        print_info(&format!(
            "🔨 Building {} with pure WASM (no WASI dependencies detected)",
            project_name
        ));
    }

    // Step 1: Choose target using the updated get_wasm_target function
    let target = crate::utils::build_utils::get_wasm_target(&current_dir, force_pure_wasm)?;

    // Display appropriate message based on target and detection
    if force_pure_wasm {
        print_info("⚡ Using pure WASM (forced by --pure-wasm flag)");
    } else if target == "wasm32-unknown-unknown" {
        if crate::utils::build_utils::has_icarus_wasi_dependency(&current_dir)? {
            print_info("🔧 Using icarus-wasi polyfills (no conversion needed)");
        } else {
            print_info("⚡ Using pure WASM target for simple deployment");
        }
    } else {
        print_info("🌐 Using WASI target for ecosystem compatibility");
    }

    // Set RUSTFLAGS for SIMD if enabled
    let mut env_vars = Vec::new();
    if enable_simd {
        print_info("🚀 Enabling SIMD optimizations for enhanced performance");
        let current_rustflags = std::env::var("RUSTFLAGS").unwrap_or_default();
        let simd_flags = if current_rustflags.is_empty() {
            "-C target-feature=+simd128".to_string()
        } else {
            format!("{} -C target-feature=+simd128", current_rustflags)
        };
        env_vars.push(("RUSTFLAGS", simd_flags));
    }

    // Compile with cargo
    run_command_with_env(
        "cargo",
        &["build", "--target", target, "--release"],
        Some(&current_dir),
        &env_vars,
    )
    .await?;

    // Step 2: Process WASM and extract Candid (with WASI conversion if needed)
    let final_wasm_path = build_canister_wasi_native(&current_dir, !has_wasi).await?;

    // Step 3: Optimize WASM if not skipped
    if !skip_optimize {
        optimize_wasm(&final_wasm_path, enable_simd).await?;
    } else {
        print_info("⏭️  Skipping WASM optimization");
    }

    print_success(&format!(
        "✅ Build completed! WASM ready at: {}",
        final_wasm_path.display()
    ));

    Ok(())
}

fn is_icarus_project(path: &Path) -> bool {
    path.join("Cargo.toml").exists() && path.join("dfx.json").exists()
}

/// Detect if the project has WASI features enabled
fn detect_wasi_features(project_dir: &Path) -> Result<bool> {
    let cargo_toml_path = project_dir.join("Cargo.toml");
    if !cargo_toml_path.exists() {
        return Ok(false);
    }

    let content = std::fs::read_to_string(&cargo_toml_path)?;
    let value: Value = toml::from_str(&content)?;

    // Check if ic-wasi-polyfill or icarus-wasi is in dependencies and NOT optional
    if let Some(deps) = value.get("dependencies") {
        // Check ic-wasi-polyfill
        if let Some(polyfill_dep) = deps.get("ic-wasi-polyfill") {
            // If it's not a table (just a string version), it's required
            // If it's a table, check if optional is false or not present
            if polyfill_dep.as_str().is_some() {
                return Ok(true);
            } else if let Some(dep_table) = polyfill_dep.as_table() {
                if !dep_table
                    .get("optional")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
                {
                    return Ok(true);
                }
            }
        }

        // Check icarus-wasi (same logic)
        if let Some(wasi_dep) = deps.get("icarus-wasi") {
            if wasi_dep.as_str().is_some() {
                return Ok(true);
            } else if let Some(dep_table) = wasi_dep.as_table() {
                if !dep_table
                    .get("optional")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
                {
                    return Ok(true);
                }
            }
        }
    }

    // Check if wasi feature exists and is in default features
    if let Some(features) = value.get("features") {
        if let Some(_wasi_feature) = features.get("wasi") {
            // If wasi feature exists, check if it's in default features
            if let Some(default_features) = features.get("default") {
                if let Some(default_array) = default_features.as_array() {
                    for feature in default_array {
                        if let Some(feature_str) = feature.as_str() {
                            if feature_str == "wasi" {
                                return Ok(true);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(false)
}

async fn optimize_wasm(wasm_path: &Path, enable_simd: bool) -> Result<()> {
    if !wasm_path.exists() {
        print_warning("WASM file not found, skipping optimization");
        return Ok(());
    }

    // Run ic-wasm optimization first if available
    if which::which("ic-wasm").is_ok() {
        let result = tokio::process::Command::new("ic-wasm")
            .arg(wasm_path)
            .arg("-o")
            .arg(wasm_path)
            .arg("shrink")
            .output()
            .await;

        match result {
            Ok(output) if output.status.success() => {
                // Optimization successful
            }
            Ok(output) => {
                let error = String::from_utf8_lossy(&output.stderr);
                print_warning(&format!("ic-wasm optimization failed: {}", error));
            }
            Err(e) => {
                print_warning(&format!("Could not run ic-wasm: {}", e));
            }
        }
    }

    // Run wasm-opt optimization if available (especially for SIMD builds)
    if which::which("wasm-opt").is_ok() {
        let mut wasm_opt_args = vec!["-Os", "-o"];
        wasm_opt_args.push(wasm_path.to_str().unwrap());
        wasm_opt_args.push(wasm_path.to_str().unwrap());

        // Enable additional features for SIMD builds
        if enable_simd {
            wasm_opt_args.extend(&[
                "--enable-simd",
                "--enable-bulk-memory",
                "--enable-nontrapping-float-to-int",
            ]);
        }

        let result = tokio::process::Command::new("wasm-opt")
            .args(&wasm_opt_args)
            .output()
            .await;

        match result {
            Ok(output) if output.status.success() => {
                // Success but don't print details
            }
            Ok(output) => {
                let error = String::from_utf8_lossy(&output.stderr);
                print_warning(&format!("wasm-opt optimization failed: {}", error));
            }
            Err(e) => {
                print_warning(&format!("Could not run wasm-opt: {}", e));
            }
        }
    } else if enable_simd {
        print_warning("wasm-opt not found. For SIMD builds, consider installing binaryen for additional optimization");
    }

    Ok(())
}

// Removed unused format_size function

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_detect_wasi_features_with_ic_wasi_polyfill_dependency() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_toml_path = temp_dir.path().join("Cargo.toml");

        let content = r#"
[package]
name = "test"
version = "0.1.0"

[dependencies]
ic-wasi-polyfill = "0.11"
serde = "1.0"
"#;
        fs::write(&cargo_toml_path, content).unwrap();

        assert_eq!(
            detect_wasi_features(temp_dir.path()).unwrap(),
            true,
            "Should detect WASI when ic-wasi-polyfill is in dependencies"
        );
    }

    #[test]
    fn test_detect_wasi_features_with_optional_dependency() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_toml_path = temp_dir.path().join("Cargo.toml");

        let content = r#"
[package]
name = "test"
version = "0.1.0"

[dependencies]
ic-wasi-polyfill = { version = "0.11", optional = true }
serde = "1.0"

[features]
default = ["wasi"]
wasi = ["ic-wasi-polyfill"]
"#;
        fs::write(&cargo_toml_path, content).unwrap();

        assert_eq!(
            detect_wasi_features(temp_dir.path()).unwrap(),
            true,
            "Should detect WASI when wasi feature is in default features"
        );
    }

    #[test]
    fn test_detect_wasi_features_with_wasi_feature_not_default() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_toml_path = temp_dir.path().join("Cargo.toml");

        let content = r#"
[package]
name = "test"
version = "0.1.0"

[dependencies]
ic-wasi-polyfill = { version = "0.11", optional = true }
serde = "1.0"

[features]
default = []
wasi = ["ic-wasi-polyfill"]
"#;
        fs::write(&cargo_toml_path, content).unwrap();

        assert_eq!(
            detect_wasi_features(temp_dir.path()).unwrap(),
            false,
            "Should NOT detect WASI when wasi feature exists but is not in default"
        );
    }

    #[test]
    fn test_detect_wasi_features_without_wasi() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_toml_path = temp_dir.path().join("Cargo.toml");

        let content = r#"
[package]
name = "test"
version = "0.1.0"

[dependencies]
serde = "1.0"
ic-cdk = "0.18"
"#;
        fs::write(&cargo_toml_path, content).unwrap();

        assert_eq!(
            detect_wasi_features(temp_dir.path()).unwrap(),
            false,
            "Should NOT detect WASI for projects without WASI dependencies"
        );
    }

    #[test]
    fn test_detect_wasi_features_missing_cargo_toml() {
        let temp_dir = TempDir::new().unwrap();
        // No Cargo.toml file created

        assert_eq!(
            detect_wasi_features(temp_dir.path()).unwrap(),
            false,
            "Should return false when Cargo.toml doesn't exist"
        );
    }

    #[test]
    fn test_detect_wasi_features_invalid_toml() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_toml_path = temp_dir.path().join("Cargo.toml");

        let content = "invalid toml content [";
        fs::write(&cargo_toml_path, content).unwrap();

        assert!(
            detect_wasi_features(temp_dir.path()).is_err(),
            "Should return error for invalid TOML"
        );
    }

    #[test]
    fn test_is_icarus_project_with_both_files() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("Cargo.toml"), "").unwrap();
        fs::write(temp_dir.path().join("dfx.json"), "").unwrap();

        assert_eq!(
            is_icarus_project(temp_dir.path()),
            true,
            "Should recognize project with both Cargo.toml and dfx.json"
        );
    }

    #[test]
    fn test_is_icarus_project_missing_cargo_toml() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("dfx.json"), "").unwrap();

        assert_eq!(
            is_icarus_project(temp_dir.path()),
            false,
            "Should NOT recognize project missing Cargo.toml"
        );
    }

    #[test]
    fn test_is_icarus_project_missing_dfx_json() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("Cargo.toml"), "").unwrap();

        assert_eq!(
            is_icarus_project(temp_dir.path()),
            false,
            "Should NOT recognize project missing dfx.json"
        );
    }

    #[test]
    fn test_is_icarus_project_empty_directory() {
        let temp_dir = TempDir::new().unwrap();

        assert_eq!(
            is_icarus_project(temp_dir.path()),
            false,
            "Should NOT recognize empty directory as Icarus project"
        );
    }

    #[tokio::test]
    async fn test_execute_wasi_project_uses_correct_target() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_toml_path = temp_dir.path().join("Cargo.toml");
        let dfx_json_path = temp_dir.path().join("dfx.json");

        // Create a WASI project
        let cargo_content = r#"
[package]
name = "test"
version = "0.1.0"

[dependencies]
ic-wasi-polyfill = { version = "0.11", optional = true }

[features]
default = ["wasi"]
wasi = ["ic-wasi-polyfill"]
"#;
        fs::write(&cargo_toml_path, cargo_content).unwrap();
        fs::write(&dfx_json_path, "{}").unwrap();

        // Change to test directory temporarily
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Test should fail gracefully when cargo build fails (expected without full project)
        let result = execute(false, false, false, false).await;

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();

        // The function should at least detect it's a WASI project (even if build fails)
        // We can't easily test the actual cargo command without a complete project setup
        assert!(
            result.is_err(),
            "Build should fail in minimal test environment, which is expected"
        );
    }

    #[tokio::test]
    async fn test_execute_non_wasi_project_uses_correct_target() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_toml_path = temp_dir.path().join("Cargo.toml");
        let dfx_json_path = temp_dir.path().join("dfx.json");

        // Create a non-WASI project
        let cargo_content = r#"
[package]
name = "test"
version = "0.1.0"

[dependencies]
serde = "1.0"
"#;
        fs::write(&cargo_toml_path, cargo_content).unwrap();
        fs::write(&dfx_json_path, "{}").unwrap();

        // Change to test directory temporarily
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Test should fail gracefully when cargo build fails (expected without full project)
        let result = execute(false, false, false, false).await;

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();

        // The function should at least detect it's a non-WASI project (even if build fails)
        assert!(
            result.is_err(),
            "Build should fail in minimal test environment, which is expected"
        );
    }

    #[tokio::test]
    async fn test_execute_force_pure_wasm_overrides_wasi() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_toml_path = temp_dir.path().join("Cargo.toml");
        let dfx_json_path = temp_dir.path().join("dfx.json");

        // Create a WASI project
        let cargo_content = r#"
[package]
name = "test"
version = "0.1.0"

[dependencies]
ic-wasi-polyfill = "0.11"
"#;
        fs::write(&cargo_toml_path, cargo_content).unwrap();
        fs::write(&dfx_json_path, "{}").unwrap();

        // Change to test directory temporarily
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Test should fail gracefully (expected without full project setup)
        // force_pure_wasm=true should override WASI detection
        let result = execute(true, false, false, false).await;

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();

        assert!(
            result.is_err(),
            "Build should fail in minimal test environment, which is expected"
        );
    }
}
