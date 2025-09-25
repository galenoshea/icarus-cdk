//! WASM binary performance analysis command

use super::*;
use anyhow::{anyhow, Result};
use colored::Colorize;
use std::fs;
use std::path::PathBuf;

/// Execute WASM analysis command
pub async fn execute(wasm_path: Option<String>, memory: bool, instructions: bool) -> Result<()> {
    println!(
        "{}",
        "🔬 Analyzing WASM Performance Characteristics"
            .bright_cyan()
            .bold()
    );

    // Find WASM file
    let wasm_file = find_wasm_file(wasm_path)?;
    println!(
        "📁 Analyzing: {}",
        wasm_file.display().to_string().bright_yellow()
    );
    println!();

    // Read WASM binary
    let wasm_bytes =
        fs::read(&wasm_file).map_err(|e| anyhow!("Failed to read WASM file: {}", e))?;

    println!("📊 Basic Analysis");
    analyze_basic_properties(&wasm_bytes, &wasm_file)?;

    if memory {
        println!("\n🧠 Memory Analysis");
        analyze_memory_characteristics(&wasm_bytes)?;
    }

    if instructions {
        println!("\n⚙️  Instruction Analysis");
        analyze_instruction_patterns(&wasm_bytes)?;
    }

    println!("\n💡 Performance Recommendations");
    provide_optimization_recommendations(&wasm_bytes, &wasm_file)?;

    Ok(())
}

/// Find WASM file to analyze
fn find_wasm_file(wasm_path: Option<String>) -> Result<PathBuf> {
    if let Some(path) = wasm_path {
        let file = PathBuf::from(path);
        if !file.exists() {
            return Err(anyhow!("WASM file not found: {}", file.display()));
        }
        return Ok(file);
    }

    // Try to find WASM file in common locations
    let current_dir = std::env::current_dir()?;
    let candidates = vec![
        current_dir
            .join("target")
            .join("wasm32-unknown-unknown")
            .join("release")
            .join("*.wasm"),
        current_dir
            .join("target")
            .join("wasm32-unknown-unknown")
            .join("debug")
            .join("*.wasm"),
        current_dir.join("*.wasm"),
    ];

    // Look for .wasm files
    for pattern_path in candidates {
        let parent = pattern_path.parent().unwrap_or(&current_dir);
        if parent.exists() {
            if let Ok(entries) = fs::read_dir(parent) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map(|ext| ext == "wasm").unwrap_or(false) {
                        println!("🔍 Found WASM file: {}", path.display());
                        return Ok(path);
                    }
                }
            }
        }
    }

    Err(anyhow!(
        "No WASM file found. Please specify --wasm-path or run from a project with built WASM output"
    ))
}

/// Analyze basic WASM properties
fn analyze_basic_properties(wasm_bytes: &[u8], wasm_file: &PathBuf) -> Result<()> {
    let size = wasm_bytes.len();
    println!(
        "   Binary size:      {}",
        utils::format_bytes(size as u64).bright_cyan()
    );

    // Check for compressed version
    let compressed_file = wasm_file.with_extension("wasm.gz");
    if compressed_file.exists() {
        if let Ok(compressed_bytes) = fs::read(&compressed_file) {
            let compressed_size = compressed_bytes.len();
            let compression_ratio = compressed_size as f64 / size as f64;
            println!(
                "   Compressed size:  {} ({:.1}% of original)",
                utils::format_bytes(compressed_size as u64).bright_green(),
                (compression_ratio * 100.0).to_string().bright_yellow()
            );
        }
    }

    // Analyze WASM header
    if wasm_bytes.len() >= 8 {
        let magic = &wasm_bytes[0..4];
        let version = &wasm_bytes[4..8];

        if magic == b"\0asm" {
            let version_num = u32::from_le_bytes([version[0], version[1], version[2], version[3]]);
            println!(
                "   WASM version:     {}",
                version_num.to_string().bright_blue()
            );
        }
    }

    // Estimate optimization level based on size patterns
    estimate_optimization_level(size);

    Ok(())
}

/// Analyze memory usage characteristics
fn analyze_memory_characteristics(wasm_bytes: &[u8]) -> Result<()> {
    // Parse WASM sections to analyze memory usage
    let sections = parse_wasm_sections(wasm_bytes)?;

    for section in sections {
        match section.section_type {
            WasmSectionType::Memory => {
                println!("   Memory sections:  Found");
                analyze_memory_section(&section.data);
            }
            WasmSectionType::Data => {
                println!(
                    "   Data sections:    {} bytes",
                    utils::format_bytes(section.data.len() as u64).bright_cyan()
                );
            }
            WasmSectionType::Global => {
                println!("   Global variables: Found");
            }
            _ => {}
        }
    }

    // Provide memory optimization suggestions
    println!("   💡 Memory tips:");
    println!("      • Use stable memory for persistent data");
    println!("      • Minimize global variables");
    println!("      • Consider memory compaction strategies");

    Ok(())
}

/// Analyze memory section data
fn analyze_memory_section(data: &[u8]) {
    if data.is_empty() {
        return;
    }

    // Simple analysis of memory section
    println!(
        "      Memory data:    {} bytes",
        utils::format_bytes(data.len() as u64).bright_cyan()
    );

    // Look for memory limits (simplified parsing)
    if data.len() >= 2 {
        let memory_count = data[0];
        println!("      Memory count:   {}", memory_count);

        if memory_count > 0 && data.len() >= 4 {
            // Parse limits (simplified)
            let has_max = (data[1] & 0x01) != 0;
            if has_max {
                println!("      Has max limit:  Yes");
            } else {
                println!("      Has max limit:  No");
            }
        }
    }
}

/// Analyze instruction patterns
fn analyze_instruction_patterns(wasm_bytes: &[u8]) -> Result<()> {
    let sections = parse_wasm_sections(wasm_bytes)?;

    for section in sections {
        if section.section_type == WasmSectionType::Code {
            println!(
                "   Code section:     {} bytes",
                utils::format_bytes(section.data.len() as u64).bright_cyan()
            );

            // Analyze instruction density
            let estimated_instructions = section.data.len() / 2; // Rough estimate
            println!(
                "   Est. instructions: ~{}",
                estimated_instructions.to_string().bright_yellow()
            );

            // Look for performance patterns
            analyze_performance_patterns(&section.data);
        }
    }

    Ok(())
}

/// Estimate optimization level based on binary size
fn estimate_optimization_level(size: usize) {
    let optimization_level = match size {
        0..=50_000 => "High (likely -O3 or -Oz)",
        50_001..=200_000 => "Medium (likely -O2)",
        200_001..=500_000 => "Low (likely -O1)",
        _ => "None or Debug (likely -O0)",
    };

    println!(
        "   Optimization:     {}",
        optimization_level.bright_magenta()
    );
}

/// Analyze performance patterns in code section
fn analyze_performance_patterns(code_data: &[u8]) {
    // Look for common performance indicators
    let mut loop_count = 0;
    let mut call_count = 0;
    let mut memory_ops = 0;

    // Simple pattern matching (this is a simplified analysis)
    for window in code_data.windows(2) {
        match window {
            [0x03, _] => loop_count += 1, // Loop opcode
            [0x10, _] => call_count += 1, // Call opcode
            [0x28, _] | [0x29, _] | [0x2a, _] | [0x2b, _] => memory_ops += 1, // Memory load/store
            _ => {}
        }
    }

    if loop_count > 0 {
        println!(
            "   Loop patterns:    ~{} detected",
            loop_count.to_string().bright_blue()
        );
    }
    if call_count > 0 {
        println!(
            "   Function calls:   ~{} detected",
            call_count.to_string().bright_green()
        );
    }
    if memory_ops > 0 {
        println!(
            "   Memory ops:       ~{} detected",
            memory_ops.to_string().bright_yellow()
        );
    }
}

/// Provide optimization recommendations
fn provide_optimization_recommendations(wasm_bytes: &[u8], wasm_file: &PathBuf) -> Result<()> {
    let size = wasm_bytes.len();

    // Size-based recommendations
    if size > 1_000_000 {
        println!("   🔴 Large binary size (>1MB):");
        println!("      • Consider code splitting");
        println!("      • Remove unused dependencies");
        println!("      • Enable LTO (Link Time Optimization)");
    } else if size > 500_000 {
        println!("   🟡 Moderate binary size (>500KB):");
        println!("      • Review dependency usage");
        println!("      • Consider wasm-opt optimization");
    } else {
        println!("   ✅ Good binary size (<500KB)");
    }

    // Compression recommendations
    let compressed_file = wasm_file.with_extension("wasm.gz");
    if !compressed_file.exists() {
        println!("   💡 Consider gzip compression for deployment");
        println!("      • Can reduce size by 60-80%");
        println!("      • Most WASM runtimes support compressed binaries");
    }

    // ICP-specific recommendations
    println!("   🌐 ICP-specific optimizations:");
    println!("      • Use stable memory for persistence");
    println!("      • Minimize inter-canister calls");
    println!("      • Implement efficient serialization");
    println!("      • Consider Candid interface optimization");

    // General performance recommendations
    println!("   ⚡ General performance:");
    println!("      • Profile with 'icarus profile canister'");
    println!("      • Use release builds for production");
    println!("      • Monitor cycles consumption");

    Ok(())
}

/// Simple WASM section parser
#[derive(Debug, PartialEq)]
enum WasmSectionType {
    Custom,
    Type,
    Import,
    Function,
    Table,
    Memory,
    Global,
    Export,
    Start,
    Element,
    Code,
    Data,
    DataCount,
    Unknown,
}

struct WasmSection {
    section_type: WasmSectionType,
    data: Vec<u8>,
}

/// Parse WASM sections (simplified parser)
fn parse_wasm_sections(wasm_bytes: &[u8]) -> Result<Vec<WasmSection>> {
    let mut sections = Vec::new();

    if wasm_bytes.len() < 8 {
        return Err(anyhow!("Invalid WASM file: too short"));
    }

    // Skip magic number and version
    let mut offset = 8;

    while offset < wasm_bytes.len() {
        if offset + 2 >= wasm_bytes.len() {
            break;
        }

        let section_type_byte = wasm_bytes[offset];
        let section_type = match section_type_byte {
            0 => WasmSectionType::Custom,
            1 => WasmSectionType::Type,
            2 => WasmSectionType::Import,
            3 => WasmSectionType::Function,
            4 => WasmSectionType::Table,
            5 => WasmSectionType::Memory,
            6 => WasmSectionType::Global,
            7 => WasmSectionType::Export,
            8 => WasmSectionType::Start,
            9 => WasmSectionType::Element,
            10 => WasmSectionType::Code,
            11 => WasmSectionType::Data,
            12 => WasmSectionType::DataCount,
            _ => WasmSectionType::Unknown,
        };

        offset += 1;

        // Parse section size (simplified LEB128 decoding)
        let (section_size, size_bytes) = decode_leb128(&wasm_bytes[offset..])?;
        offset += size_bytes;

        if offset + section_size > wasm_bytes.len() {
            break;
        }

        let section_data = wasm_bytes[offset..offset + section_size].to_vec();
        sections.push(WasmSection {
            section_type,
            data: section_data,
        });

        offset += section_size;
    }

    Ok(sections)
}

/// Simplified LEB128 decoder for section sizes
fn decode_leb128(data: &[u8]) -> Result<(usize, usize)> {
    let mut result = 0;
    let mut shift = 0;
    let mut bytes_read = 0;

    for &byte in data.iter().take(5) {
        // LEB128 u32 max 5 bytes
        bytes_read += 1;
        result |= ((byte & 0x7F) as usize) << shift;

        if byte & 0x80 == 0 {
            return Ok((result, bytes_read));
        }

        shift += 7;
        if shift >= 32 {
            return Err(anyhow!("LEB128 decode error: value too large"));
        }
    }

    Err(anyhow!("LEB128 decode error: incomplete data"))
}
