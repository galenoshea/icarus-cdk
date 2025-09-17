use anyhow::Result;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

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
        // WASI-Native: Default to WASI for ecosystem compatibility
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

/// Ensure wasi2ic is available and download if needed
pub async fn ensure_wasi2ic() -> Result<std::path::PathBuf> {
    // Check if wasi2ic is already in PATH
    if let Ok(path) = which::which("wasi2ic") {
        return Ok(path);
    }

    // Create tools directory
    let home_dir =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    let tools_dir = home_dir.join(".icarus").join("tools");
    std::fs::create_dir_all(&tools_dir)?;

    let wasi2ic_path = tools_dir.join("wasi2ic");

    // Check if we already downloaded it
    if wasi2ic_path.exists() {
        return Ok(wasi2ic_path);
    }

    // Download wasi2ic binary
    crate::utils::print_info("Downloading wasi2ic tool for WASI support...");

    // Determine download URL based on platform
    let (url, filename) = if cfg!(target_os = "macos") {
        (
            "https://github.com/wasm-forge/wasi2ic/releases/latest/download/wasi2ic-macos",
            "wasi2ic",
        )
    } else if cfg!(target_os = "linux") {
        (
            "https://github.com/wasm-forge/wasi2ic/releases/latest/download/wasi2ic-linux",
            "wasi2ic",
        )
    } else if cfg!(target_os = "windows") {
        (
            "https://github.com/wasm-forge/wasi2ic/releases/latest/download/wasi2ic-windows.exe",
            "wasi2ic.exe",
        )
    } else {
        anyhow::bail!("Unsupported platform for wasi2ic download");
    };

    // Download the binary
    let response = reqwest::get(url).await?;
    let bytes = response.bytes().await?;

    let final_path = tools_dir.join(filename);
    std::fs::write(&final_path, bytes)?;

    // Make executable on Unix-like systems
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&final_path)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&final_path, perms)?;
    }

    crate::utils::print_success("wasi2ic downloaded successfully");
    Ok(final_path)
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

/// Convert WASI WASM to IC-compatible WASM using wasi2ic with caching
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

    let wasi2ic_path = ensure_wasi2ic().await?;

    crate::utils::print_info("Converting WASI WASM to IC-compatible format...");

    // Ensure output directory exists
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let output = tokio::process::Command::new(&wasi2ic_path)
        .arg(wasi_wasm_path)
        .arg(output_path)
        .output()
        .await?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("wasi2ic conversion failed: {}", error);
    }

    // Save cache metadata
    save_conversion_cache(project_dir, wasi_wasm_path, output_path)?;

    crate::utils::print_success("WASI WASM converted successfully");
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
    let use_wasi = target == "wasm32-wasip1";

    if use_wasi {
        crate::utils::print_info(
            "🧬 Using WASI-Native architecture for maximum ecosystem compatibility",
        );
    } else {
        crate::utils::print_info("⚡ Using pure WASM (advanced users only)");
    }

    // Step 1: Compile to WASM
    let wasm_path = get_wasm_path(project_dir, &project_name, target);

    // Step 2: If WASI, convert to IC-compatible WASM
    if use_wasi {
        // Final output goes to standard location for compatibility with tooling
        let final_wasm_path = get_wasm_path(project_dir, &project_name, "wasm32-unknown-unknown");
        convert_wasi_to_ic(project_dir, &wasm_path, &final_wasm_path).await?;
        Ok(final_wasm_path)
    } else {
        Ok(wasm_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_wasi_native_default() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_toml = temp_dir.path().join("Cargo.toml");

        // Test default WASI-Native behavior (any project)
        let content = r#"
[package]
name = "test"
version = "0.1.0"

[dependencies]
serde = "1.0"
"#;
        fs::write(&cargo_toml, content).unwrap();

        // WASI should be default, even for non-ML projects
        assert_eq!(
            get_wasm_target(temp_dir.path(), false).unwrap(),
            "wasm32-wasip1"
        );

        // Force pure WASM should override
        assert_eq!(
            get_wasm_target(temp_dir.path(), true).unwrap(),
            "wasm32-unknown-unknown"
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
}
