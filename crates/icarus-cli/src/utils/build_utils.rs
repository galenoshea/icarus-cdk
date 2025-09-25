use anyhow::Result;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

// Import icarus-wasi for in-memory WASI conversion
use icarus_wasi;

/// Check if icarus-wasi dependency is present (polyfill approach)
pub fn has_icarus_wasi_dependency(project_dir: &Path) -> Result<bool> {
    let cargo_toml_path = project_dir.join("Cargo.toml");
    if !cargo_toml_path.exists() {
        return Ok(false);
    }

    let content = std::fs::read_to_string(&cargo_toml_path)?;
    let value: toml::Value = toml::from_str(&content)?;

    if let Some(dependencies) = value.get("dependencies") {
        if dependencies.get("icarus-wasi").is_some() {
            return Ok(true);
        }
    }

    Ok(false)
}

/// Check if pure WASM mode is explicitly disabled (WASI-Native by default)
pub fn is_pure_wasm_enabled(project_dir: &Path) -> Result<bool> {
    let cargo_toml_path = project_dir.join("Cargo.toml");
    if !cargo_toml_path.exists() {
        return Ok(false);
    }

    let content = std::fs::read_to_string(&cargo_toml_path)?;
    let value: toml::Value = toml::from_str(&content)?;

    if let Some(package) = value.get("package") {
        if let Some(metadata) = package.get("metadata") {
            if let Some(icarus) = metadata.get("icarus") {
                if let Some(wasi_mode) = icarus.get("wasi_mode") {
                    return Ok(wasi_mode.as_str() == Some("never"));
                }
            }
        }
    }

    Ok(false)
}

/// Get WASM target with WASI as default (WASI-Native architecture)
pub fn get_wasm_target(project_dir: &Path, force_pure_wasm: bool) -> Result<&'static str> {
    // Force pure WASM if explicitly requested
    if force_pure_wasm {
        return Ok("wasm32-unknown-unknown");
    }

    // Check if pure WASM is explicitly enabled in metadata
    if is_pure_wasm_enabled(project_dir)? {
        Ok("wasm32-unknown-unknown")
    } else {
        // WASI-Native: Default to WASI for ecosystem compatibility (with conversion)
        // This includes projects with icarus-wasi dependency which still need conversion
        Ok("wasm32-wasip1")
    }
}

/// Get the target directory for the specified WASM target
pub fn get_target_dir(project_dir: &Path, target: &str) -> std::path::PathBuf {
    project_dir.join("target").join(target).join("release")
}

/// Get the WASM file path for a project
pub fn get_wasm_path(project_dir: &Path, project_name: &str, target: &str) -> std::path::PathBuf {
    let wasm_name = project_name.replace('-', "_");
    get_target_dir(project_dir, target).join(format!("{}.wasm", wasm_name))
}

/// Get project name from Cargo.toml
pub fn get_project_name(project_dir: &Path) -> Result<String> {
    let cargo_toml = project_dir.join("Cargo.toml");
    let content = std::fs::read_to_string(&cargo_toml)?;
    let toml: toml::Value = toml::from_str(&content)?;

    toml.get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow::anyhow!("Could not find package name in Cargo.toml"))
}

/// Get cache directory for wasi2ic conversions
fn get_cache_dir(project_dir: &Path) -> std::path::PathBuf {
    project_dir
        .join("target")
        .join(".icarus-cache")
        .join("wasi2ic")
}

/// Calculate SHA256 hash of a file for cache key
fn calculate_file_hash(file_path: &Path) -> Result<String> {
    let contents = fs::read(file_path)?;
    let mut hasher = Sha256::new();
    hasher.update(&contents);
    Ok(format!("{:x}", hasher.finalize()))
}

/// Check if conversion is cached and still valid
fn is_conversion_cached(
    project_dir: &Path,
    wasi_wasm_path: &Path,
    output_path: &Path,
) -> Result<bool> {
    // Check if output exists
    if !output_path.exists() {
        return Ok(false);
    }

    // Calculate hash of current WASI WASM
    let current_hash = calculate_file_hash(wasi_wasm_path)?;

    // Check if cached hash matches
    let cache_dir = get_cache_dir(project_dir);
    let hash_file = cache_dir.join(format!(
        "{}.hash",
        output_path.file_stem().unwrap().to_string_lossy()
    ));

    if hash_file.exists() {
        if let Ok(cached_hash) = fs::read_to_string(&hash_file) {
            return Ok(cached_hash.trim() == current_hash);
        }
    }

    Ok(false)
}

/// Save conversion cache metadata
fn save_conversion_cache(
    project_dir: &Path,
    wasi_wasm_path: &Path,
    output_path: &Path,
) -> Result<()> {
    let cache_dir = get_cache_dir(project_dir);
    fs::create_dir_all(&cache_dir)?;

    let current_hash = calculate_file_hash(wasi_wasm_path)?;
    let hash_file = cache_dir.join(format!(
        "{}.hash",
        output_path.file_stem().unwrap().to_string_lossy()
    ));

    fs::write(hash_file, current_hash)?;
    Ok(())
}

/// Convert WASI WASM to IC-compatible WASM using in-memory wasi2ic conversion with caching
pub async fn convert_wasi_to_ic(
    project_dir: &Path,
    wasi_wasm_path: &Path,
    output_path: &Path,
) -> Result<()> {
    // Check cache first
    if is_conversion_cached(project_dir, wasi_wasm_path, output_path)? {
        crate::utils::print_success("Using cached WASI conversion");
        return Ok(());
    }

    // Read the WASI WASM bytes
    let wasi_wasm_bytes = fs::read(wasi_wasm_path)?;

    // Convert WASI to IC-compatible format using in-memory conversion
    let ic_wasm_bytes = icarus_wasi::convert_wasi_to_ic(&wasi_wasm_bytes)?;

    // Ensure output directory exists
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Write the converted WASM
    fs::write(output_path, ic_wasm_bytes)?;

    // Save cache metadata
    save_conversion_cache(project_dir, wasi_wasm_path, output_path)?;

    crate::utils::print_success("WASI conversion completed successfully");
    Ok(())
}

/// Validate that extracted Candid contains required Icarus methods
fn validate_candid_content(candid_content: &str) -> Result<()> {
    // Check if it contains a service definition
    if !candid_content.contains("service") {
        anyhow::bail!("Extracted Candid does not contain a service definition");
    }

    // Required auth methods that should be injected by the service macro
    let required_methods = ["add_user", "remove_user", "get_current_user", "get_tools"];

    for method in &required_methods {
        if !candid_content.contains(method) {
            anyhow::bail!(
                "Candid interface missing required method '{}'. This suggests the #[service] macro \
                phantom types are not working correctly. Check that export_candid!() is properly \
                configured with the __candid_export module.",
                method
            );
        }
    }

    // Check for proper init method signature
    if !candid_content.contains("principal") {
        anyhow::bail!(
            "Candid interface missing principal parameter in init method. \
            This is required for Icarus authentication system."
        );
    }

    // Candid interface validation passed
    Ok(())
}

/// Extract Candid from WASM using candid-extractor
async fn extract_candid_from_wasm_path(wasm_path: &Path, did_path: &Path) -> Result<()> {
    // Check if WASM exists
    if !wasm_path.exists() {
        anyhow::bail!("WASM file not found at {:?}", wasm_path);
    }

    // Check if candid-extractor is available
    if which::which("candid-extractor").is_err() {
        anyhow::bail!("candid-extractor not found. Install with: cargo install candid-extractor");
    }

    // Extract Candid from WASM
    let output = tokio::process::Command::new("candid-extractor")
        .arg(wasm_path)
        .output()
        .await?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Could not extract Candid: {}", error);
    }

    // Get the extracted Candid
    let candid_content = String::from_utf8_lossy(&output.stdout);

    // Validate extracted Candid content
    validate_candid_content(&candid_content)?;

    // Write validated Candid to file
    fs::write(did_path, candid_content.as_ref())?;
    // Candid interface validated and saved
    Ok(())
}

/// Build canister with WASI-Native architecture
/// Returns the path to the final IC-compatible WASM file
pub async fn build_canister_wasi_native(
    project_dir: &Path,
    force_pure_wasm: bool,
) -> Result<std::path::PathBuf> {
    let project_name = get_project_name(project_dir)?;
    let target = get_wasm_target(project_dir, force_pure_wasm)?;
    let use_wasi_conversion = target == "wasm32-wasip1";
    let has_wasi_polyfills = has_icarus_wasi_dependency(project_dir)?;

    crate::utils::print_info(&format!(
        "DEBUG: target={}, use_wasi_conversion={}, has_wasi_polyfills={}",
        target, use_wasi_conversion, has_wasi_polyfills
    ));

    // Architecture selected (WASI conversion, WASI with polyfills, or pure WASM)
    if has_wasi_polyfills && use_wasi_conversion {
        crate::utils::print_info("🔧 Using WASI target with icarus-wasi in-memory conversion");
    } else if use_wasi_conversion {
        crate::utils::print_info("🔧 Using WASI target with conversion");
    } else {
        crate::utils::print_info("🔧 Using pure WASM target");
    }

    // Step 1: Compile to WASM
    let wasm_path = get_wasm_path(project_dir, &project_name, target);

    // Step 2: If WASI conversion needed, convert to IC-compatible format FIRST, then extract Candid
    if use_wasi_conversion {
        eprintln!("DEBUG build_utils: About to call convert_wasi_to_ic");
        // Convert WASI to IC-compatible WASM IN PLACE
        convert_wasi_to_ic(project_dir, &wasm_path, &wasm_path).await?;
        eprintln!("DEBUG build_utils: convert_wasi_to_ic completed");

        // Create IC-compatible WASM path for dfx deployment (with _ic suffix)
        let wasm_name = project_name.replace('-', "_");
        let final_wasm_path =
            get_target_dir(project_dir, target).join(format!("{}_ic.wasm", wasm_name));
        if let Some(parent) = final_wasm_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::copy(&wasm_path, &final_wasm_path)?;

        // Now extract Candid from the IC-compatible WASM
        let did_path = project_dir
            .join("src")
            .join(format!("{}.did", project_name));

        if let Err(e) = extract_candid_from_wasm_path(&final_wasm_path, &did_path).await {
            crate::utils::print_warning(&format!("Could not extract Candid from IC WASM: {}", e));
            crate::utils::print_info("Continuing with existing .did file...");
        } else {
            // Candid interface extracted from IC-compatible WASM
        }

        // Return the final deployment path
        Ok(final_wasm_path)
    } else {
        // For icarus-wasi polyfills or pure WASM, create _ic.wasm for dfx compatibility
        let wasm_name = project_name.replace('-', "_");
        let final_wasm_path =
            get_target_dir(project_dir, target).join(format!("{}_ic.wasm", wasm_name));
        if let Some(parent) = final_wasm_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::copy(&wasm_path, &final_wasm_path)?;

        // Extract Candid from the WASM
        let did_path = project_dir
            .join("src")
            .join(format!("{}.did", project_name));

        if let Err(e) = extract_candid_from_wasm_path(&final_wasm_path, &did_path).await {
            crate::utils::print_warning(&format!("Could not extract Candid from WASM: {}", e));
            crate::utils::print_info("Continuing with existing .did file...");
        }

        Ok(final_wasm_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_wasm_target_selection_logic() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_toml = temp_dir.path().join("Cargo.toml");

        // Test default behavior without WASI configuration (should be pure WASM)
        let content_basic = r#"
[package]
name = "test"
version = "0.1.0"

[dependencies]
serde = "1.0"
"#;
        fs::write(&cargo_toml, content_basic).unwrap();

        // Without WASI metadata, should default to WASI for ecosystem compatibility
        assert_eq!(
            get_wasm_target(temp_dir.path(), false).unwrap(),
            "wasm32-wasip1",
            "Default behavior should be WASI for ecosystem compatibility"
        );

        // Force pure WASM should always override
        assert_eq!(
            get_wasm_target(temp_dir.path(), true).unwrap(),
            "wasm32-unknown-unknown",
            "Force pure WASM flag should override default behavior"
        );
    }

    #[test]
    fn test_pure_wasm_explicit_config() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_toml = temp_dir.path().join("Cargo.toml");

        // Test with explicit pure WASM mode
        let content_pure_wasm = r#"
[package]
name = "test"
version = "0.1.0"

[package.metadata.icarus]
wasi_mode = "never"

[dependencies]
serde = "1.0"
"#;
        fs::write(&cargo_toml, content_pure_wasm).unwrap();
        assert!(is_pure_wasm_enabled(temp_dir.path()).unwrap());
        assert_eq!(
            get_wasm_target(temp_dir.path(), false).unwrap(),
            "wasm32-unknown-unknown"
        );
    }

    #[test]
    fn test_project_name_extraction() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_toml = temp_dir.path().join("Cargo.toml");

        let content = r#"
[package]
name = "my-test-project"
version = "0.1.0"
"#;
        fs::write(&cargo_toml, content).unwrap();
        assert_eq!(
            get_project_name(temp_dir.path()).unwrap(),
            "my-test-project"
        );
    }

    #[test]
    fn test_wasm_path_generation() {
        let temp_dir = TempDir::new().unwrap();

        let wasm_path = get_wasm_path(temp_dir.path(), "my-project", "wasm32-wasip1");
        assert!(wasm_path.to_string_lossy().contains("wasm32-wasip1"));
        assert!(wasm_path.to_string_lossy().contains("my_project.wasm"));
    }

    #[test]
    fn test_cache_functionality() {
        let temp_dir = TempDir::new().unwrap();

        // Create a dummy WASI file
        let wasi_path = temp_dir.path().join("test.wasm");
        fs::write(&wasi_path, b"dummy wasm content").unwrap();

        let output_path = temp_dir.path().join("output.wasm");

        // Should not be cached initially
        assert!(!is_conversion_cached(temp_dir.path(), &wasi_path, &output_path).unwrap());

        // Create output file and save cache
        fs::write(&output_path, b"converted content").unwrap();
        save_conversion_cache(temp_dir.path(), &wasi_path, &output_path).unwrap();

        // Should now be cached
        assert!(is_conversion_cached(temp_dir.path(), &wasi_path, &output_path).unwrap());

        // Modify WASI file, should invalidate cache
        fs::write(&wasi_path, b"modified wasm content").unwrap();
        assert!(!is_conversion_cached(temp_dir.path(), &wasi_path, &output_path).unwrap());
    }

    #[test]
    fn test_wasi_detection_with_different_configurations() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_toml = temp_dir.path().join("Cargo.toml");

        // Test project with explicit WASI configuration (should be pure WASM when disabled)
        let content_wasi_disabled = r#"
[package]
name = "test"
version = "0.1.0"

[package.metadata.icarus]
wasi_mode = "never"

[dependencies]
serde = "1.0"
"#;
        fs::write(&cargo_toml, content_wasi_disabled).unwrap();
        assert!(is_pure_wasm_enabled(temp_dir.path()).unwrap());
        assert_eq!(
            get_wasm_target(temp_dir.path(), false).unwrap(),
            "wasm32-unknown-unknown",
            "Explicit WASI disabled should use pure WASM"
        );

        // Test project with WASI enabled explicitly
        let content_wasi_enabled = r#"
[package]
name = "test"
version = "0.1.0"

[package.metadata.icarus]
wasi_mode = "enabled"

[dependencies]
serde = "1.0"
"#;
        fs::write(&cargo_toml, content_wasi_enabled).unwrap();
        assert!(!is_pure_wasm_enabled(temp_dir.path()).unwrap());
        assert_eq!(
            get_wasm_target(temp_dir.path(), false).unwrap(),
            "wasm32-wasip1",
            "WASI enabled should use WASI target"
        );

        // Test project without metadata (default case)
        let content_no_metadata = r#"
[package]
name = "test"
version = "0.1.0"

[dependencies]
serde = "1.0"
"#;
        fs::write(&cargo_toml, content_no_metadata).unwrap();
        assert!(!is_pure_wasm_enabled(temp_dir.path()).unwrap());
        assert_eq!(
            get_wasm_target(temp_dir.path(), false).unwrap(),
            "wasm32-wasip1",
            "No metadata should default to WASI for ecosystem compatibility"
        );
    }

    #[test]
    fn test_wasm_path_generation_with_different_targets() {
        let temp_dir = TempDir::new().unwrap();

        // Test WASI target path generation
        let wasi_path = get_wasm_path(temp_dir.path(), "my-project", "wasm32-wasip1");
        assert!(wasi_path.to_string_lossy().contains("wasm32-wasip1"));
        assert!(wasi_path.to_string_lossy().contains("my_project.wasm"));
        assert!(wasi_path.to_string_lossy().contains("release"));

        // Test pure WASM target path generation
        let pure_wasm_path = get_wasm_path(temp_dir.path(), "my-project", "wasm32-unknown-unknown");
        assert!(pure_wasm_path
            .to_string_lossy()
            .contains("wasm32-unknown-unknown"));
        assert!(pure_wasm_path.to_string_lossy().contains("my_project.wasm"));
        assert!(pure_wasm_path.to_string_lossy().contains("release"));

        // Test with hyphenated project name (should be converted to underscores)
        let hyphenated_path =
            get_wasm_path(temp_dir.path(), "my-hyphenated-project", "wasm32-wasip1");
        assert!(hyphenated_path
            .to_string_lossy()
            .contains("my_hyphenated_project.wasm"));
        assert!(!hyphenated_path
            .to_string_lossy()
            .contains("my-hyphenated-project.wasm"));
    }

    #[test]
    fn test_force_pure_wasm_override() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_toml = temp_dir.path().join("Cargo.toml");

        // Test that force_pure_wasm overrides any configuration
        let content_wasi_enabled = r#"
[package]
name = "test"
version = "0.1.0"

[package.metadata.icarus]
wasi_mode = "enabled"

[dependencies]
ic-wasi-polyfill = "0.11"
"#;
        fs::write(&cargo_toml, content_wasi_enabled).unwrap();

        // Without force flag, should use WASI
        assert_eq!(
            get_wasm_target(temp_dir.path(), false).unwrap(),
            "wasm32-wasip1",
            "Without force flag should respect WASI configuration"
        );

        // With force flag, should use pure WASM regardless of configuration
        assert_eq!(
            get_wasm_target(temp_dir.path(), true).unwrap(),
            "wasm32-unknown-unknown",
            "Force flag should override any WASI configuration"
        );
    }

    #[test]
    fn test_project_name_extraction_edge_cases() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_toml = temp_dir.path().join("Cargo.toml");

        // Test with complex project name
        let content = r#"
[package]
name = "my-complex_project-name123"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = "1.0"
"#;
        fs::write(&cargo_toml, content).unwrap();
        assert_eq!(
            get_project_name(temp_dir.path()).unwrap(),
            "my-complex_project-name123"
        );

        // Test missing Cargo.toml
        let nonexistent_dir = temp_dir.path().join("nonexistent");
        assert!(get_project_name(&nonexistent_dir).is_err());

        // Test malformed Cargo.toml
        let malformed_content = "invalid toml content [";
        fs::write(&cargo_toml, malformed_content).unwrap();
        assert!(get_project_name(temp_dir.path()).is_err());
    }

    #[test]
    fn test_target_directory_structure() {
        let temp_dir = TempDir::new().unwrap();

        // Test target directory generation for different targets
        let wasi_target_dir = get_target_dir(temp_dir.path(), "wasm32-wasip1");
        assert!(wasi_target_dir.to_string_lossy().contains("target"));
        assert!(wasi_target_dir.to_string_lossy().contains("wasm32-wasip1"));
        assert!(wasi_target_dir.to_string_lossy().contains("release"));

        let pure_wasm_target_dir = get_target_dir(temp_dir.path(), "wasm32-unknown-unknown");
        assert!(pure_wasm_target_dir.to_string_lossy().contains("target"));
        assert!(pure_wasm_target_dir
            .to_string_lossy()
            .contains("wasm32-unknown-unknown"));
        assert!(pure_wasm_target_dir.to_string_lossy().contains("release"));

        // Both should be under the project directory
        assert!(wasi_target_dir.starts_with(temp_dir.path()));
        assert!(pure_wasm_target_dir.starts_with(temp_dir.path()));
    }

    #[test]
    fn test_cache_directory_structure() {
        let temp_dir = TempDir::new().unwrap();

        let cache_dir = get_cache_dir(temp_dir.path());
        assert!(cache_dir.to_string_lossy().contains("target"));
        assert!(cache_dir.to_string_lossy().contains(".icarus-cache"));
        assert!(cache_dir.to_string_lossy().contains("wasi2ic"));
        assert!(cache_dir.starts_with(temp_dir.path()));
    }
}
