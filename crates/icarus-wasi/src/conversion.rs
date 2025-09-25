//! Simple WASI to IC conversion using external wasi2ic tool
//!
//! This module provides a simple wrapper around the wasi2ic command line tool
//! to convert WASI-dependent WebAssembly modules to IC-compatible format.

use anyhow::Result;
use std::fs;
use std::process::Command;
use tempfile::NamedTempFile;

/// Convert WASI-dependent WASM bytes to IC-compatible WASM bytes
///
/// This function uses the external wasi2ic tool to perform the conversion.
/// It creates temporary files for input/output and calls wasi2ic.
///
/// # Arguments
///
/// * `wasm_bytes` - The input WASM bytes compiled for wasm32-wasip1
///
/// # Returns
///
/// * `Ok(Vec<u8>)` - IC-compatible WASM bytes with WASI imports replaced
/// * `Err(anyhow::Error)` - If conversion fails
pub fn convert_wasi_to_ic(wasm_bytes: &[u8]) -> Result<Vec<u8>> {
    eprintln!(
        "DEBUG: convert_wasi_to_ic called with {} bytes",
        wasm_bytes.len()
    );

    // Check if input has WASI imports
    let has_wasi = has_wasi_imports(wasm_bytes)?;
    eprintln!("DEBUG: Input WASM has WASI imports: {}", has_wasi);

    // Check if wasi2ic is available
    if which::which("wasi2ic").is_err() {
        anyhow::bail!("wasi2ic not found. Install with: cargo install wasi2ic");
    }

    // Create temporary files for input and output
    let input_file = NamedTempFile::new()?;
    let output_file = NamedTempFile::new()?;

    // Write input WASM to temporary file
    fs::write(input_file.path(), wasm_bytes)?;
    eprintln!("DEBUG: Written input WASM to: {:?}", input_file.path());

    // Run wasi2ic conversion
    let output = Command::new("wasi2ic")
        .arg(input_file.path())
        .arg(output_file.path())
        .output()?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("wasi2ic conversion failed: {}", error);
    }

    // Log success for debugging
    let stdout = String::from_utf8_lossy(&output.stdout);
    eprintln!("wasi2ic output: {}", stdout);

    // Read the converted WASM
    let converted_bytes = fs::read(output_file.path())?;
    eprintln!(
        "DEBUG: Read {} bytes from output file",
        converted_bytes.len()
    );

    // Check if output has WASI imports
    let output_has_wasi = has_wasi_imports(&converted_bytes)?;
    eprintln!("DEBUG: Output WASM has WASI imports: {}", output_has_wasi);

    Ok(converted_bytes)
}

/// Check if WASM bytes contain WASI imports that need conversion
///
/// This is a utility function to determine if a WASM module needs WASI conversion.
/// It can be used to optimize the build process by skipping conversion for
/// modules that don't use WASI.
///
/// # Arguments
///
/// * `wasm_bytes` - The WASM bytes to check
///
/// # Returns
///
/// * `Ok(true)` - If WASI imports are found
/// * `Ok(false)` - If no WASI imports are found
/// * `Err(anyhow::Error)` - If parsing fails
pub fn has_wasi_imports(wasm_bytes: &[u8]) -> Result<bool> {
    // Simple string-based check for WASI imports
    // This is less accurate than parsing but much simpler
    let wasm_str = String::from_utf8_lossy(wasm_bytes);
    Ok(wasm_str.contains("wasi_snapshot_preview1") || wasm_str.contains("wasi_unstable"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_wasi_imports_with_empty_bytes() {
        let empty_bytes = Vec::new();
        let result = has_wasi_imports(&empty_bytes).unwrap();
        assert!(!result, "Empty bytes should not have WASI imports");
    }

    #[test]
    fn test_has_wasi_imports_with_wasi_string() {
        let fake_wasm = b"some wasm with wasi_snapshot_preview1 import";
        let result = has_wasi_imports(fake_wasm).unwrap();
        assert!(result, "Should detect WASI imports");
    }
}
