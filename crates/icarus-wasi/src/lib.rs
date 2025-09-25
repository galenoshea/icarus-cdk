//! Self-contained WASI support for Icarus canisters
//!
//! This crate provides complete WASI support for Icarus canisters by combining:
//! 1. Build-time WASM conversion (wasm32-wasip1 → IC-compatible)
//! 2. Runtime polyfill functions for WASI system calls
//! 3. Automatic initialization through the auth!() macro
//!
//! When included as a dependency, the build process will automatically:
//!
//! 1. Build the canister with `wasm32-wasip1` target (where system calls work)
//! 2. Include ic-wasi-polyfill functions in the WASM module
//! 3. Transform the resulting WASM using wasi2ic to replace WASI imports
//! 4. Initialize WASI runtime when auth!() macro is used
//!
//! # Usage
//!
//! Add both icarus-wasi and the "wasi" feature to enable complete WASI support:
//!
//! ```toml
//! [dependencies]
//! icarus = { version = "0.8.0", features = ["canister", "wasi"] }
//! icarus-wasi = "0.8.0"
//! ```
//!
//! Then use the standard auth!() macro - WASI initialization is automatic:
//!
//! ```rust,ignore
//! use icarus::prelude::*;
//!
//! // WASI is automatically initialized when auth!() detects icarus-wasi
//! icarus::auth!();
//! icarus::mcp!();
//!
//! // Your tools can now use tokio, reqwest, std::fs, etc.
//! ```
//!
//! # Architecture
//!
//! - **Self-contained**: Includes both build-time and runtime WASI support
//! - **Automatic initialization**: Handled by auth!() macro with feature detection
//! - **Target transformation**: wasm32-wasip1 → IC-compatible WASM via wasi2ic
//! - **Runtime polyfill**: ic-wasi-polyfill provides __ic_custom_* functions
//! - **Ecosystem compatibility**: Works with tokio, reqwest, and other system libraries
//!
//! # Programmatic Usage
//!
//! For advanced use cases, you can also use the conversion functions directly:
//!
//! ```rust,ignore
//! use icarus_wasi::convert_wasi_to_ic;
//!
//! let wasi_wasm = std::fs::read("my_canister.wasm")?;
//! let ic_wasm = convert_wasi_to_ic(&wasi_wasm)?;
//! std::fs::write("my_canister_ic.wasm", ic_wasm)?;
//! ```

// Modules
pub mod conversion;

// Re-export main conversion functions for easy access
pub use conversion::{convert_wasi_to_ic, has_wasi_imports};

// Re-export ic-wasi-polyfill when feature is enabled
#[cfg(feature = "polyfill")]
pub use ic_wasi_polyfill;

/// WASI marker and build instructions
///
/// This crate serves as a marker to indicate WASI support is needed.
/// The actual WASI handling is done at build time by the Icarus CLI.
/// Marker constant indicating WASI support is enabled
pub const WASI_ENABLED: bool = true;

/// Get build instructions for WASI projects
///
/// Returns information about how this project should be built with WASI support.
/// This is primarily used by the Icarus CLI to determine build strategy.
pub fn build_info() -> WasiBuildInfo {
    WasiBuildInfo {
        needs_wasi: true,
        target: "wasm32-wasip1",
        transform_required: true,
    }
}

/// Build information for WASI projects
#[derive(Debug, Clone)]
pub struct WasiBuildInfo {
    /// Whether this project needs WASI support
    pub needs_wasi: bool,
    /// The WASM target to use for building
    pub target: &'static str,
    /// Whether transformation is required after building
    pub transform_required: bool,
}

/// Initialize WASI runtime polyfill
///
/// This macro initializes the WASI polyfill functions that replace WASI system calls
/// with IC-compatible implementations. It's called automatically by the auth!() macro
/// when the "wasi" feature is enabled.
///
/// # Examples
///
/// Basic usage (automatic via auth!() macro):
/// ```rust,ignore
/// // WASI initialization is automatic when auth!() is used
/// icarus::auth!();
/// ```
///
/// Manual usage (advanced):
/// ```rust,ignore
/// icarus_wasi::wasi_init!();  // Initialize with default seed
/// icarus_wasi::wasi_init!(&[1,2,3,4]);  // Initialize with custom seed
/// ```
#[macro_export]
macro_rules! wasi_init {
    () => {
        #[cfg(feature = "polyfill")]
        {
            // Ensure polyfill functions are linked by calling init
            static WASI_INIT: ::std::sync::Once = ::std::sync::Once::new();
            WASI_INIT.call_once(|| {
                // This call ensures the entire polyfill is linked into the binary
                $crate::ic_wasi_polyfill::init(&[0u8; 32], &[]);
            });
        }
        #[cfg(not(feature = "polyfill"))]
        {
            // No-op when polyfill not available
        }
    };
    ($seed:expr) => {
        #[cfg(feature = "polyfill")]
        {
            static WASI_INIT: ::std::sync::Once = ::std::sync::Once::new();
            WASI_INIT.call_once(|| {
                $crate::ic_wasi_polyfill::init($seed, &[]);
            });
        }
        #[cfg(not(feature = "polyfill"))]
        {
            // No-op when polyfill not available
        }
    };
    ($seed:expr, $env:expr) => {
        #[cfg(feature = "polyfill")]
        {
            static WASI_INIT: ::std::sync::Once = ::std::sync::Once::new();
            WASI_INIT.call_once(|| {
                $crate::ic_wasi_polyfill::init($seed, $env);
            });
        }
        #[cfg(not(feature = "polyfill"))]
        {
            // No-op when polyfill not available
        }
    };
}

/// Macro to mark WASI usage in your canister
///
/// This is a no-op at runtime but serves as documentation that your
/// canister uses WASI-dependent libraries.
///
/// # Example
///
/// ```rust,ignore
/// use icarus::prelude::*;
/// use icarus_wasi::wasi;
///
/// // Mark that this canister uses WASI
/// wasi!();
///
/// // Standard Icarus setup
/// auth!();
/// mcp!();
///
/// // Your tools can now use tokio, reqwest, etc.
/// #[tool]
/// pub fn fetch_data() -> String {
///     // WASI libraries work automatically
///     "Data fetched successfully!".to_string()
/// }
/// ```
#[macro_export]
macro_rules! wasi {
    () => {
        // This is a marker for WASI usage - polyfill linkage is handled by auth!() macro
        const _: () = {
            // Compile-time assertion that WASI is enabled
            const _: bool = $crate::WASI_ENABLED;
        };
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasi_enabled() {
        assert!(WASI_ENABLED);
    }

    #[test]
    fn test_build_info() {
        let info = build_info();
        assert!(info.needs_wasi);
        assert_eq!(info.target, "wasm32-wasip1");
        assert!(info.transform_required);
    }

    #[test]
    fn test_wasi_macro() {
        // Test that the macro compiles
        wasi!();
    }

    #[test]
    fn test_wasi_init_macro() {
        // Test that the init macro compiles
        wasi_init!();
        wasi_init!(&[1, 2, 3, 4]);
        wasi_init!(&[1, 2, 3, 4], &[("TEST", "value")]);
    }
}
