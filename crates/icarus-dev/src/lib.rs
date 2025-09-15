//! # Icarus Dev Tools
//!
//! Development utilities for the Icarus SDK that enhance the development experience
//! with file watching, status monitoring, and project management tools.
//!
//! ## Features
//!
//! - **File Watching**: Real-time file monitoring with intelligent debouncing
//! - **Project Status**: Comprehensive development environment status checking
//! - **Auto-rebuild**: Automatic project rebuilding and redeployment on changes
//! - **Interactive Setup**: Guided project initialization and configuration
//! - **Environment Reset**: Clean development environment reset utilities
//!
//! ## Quick Start
//!
//! ```no_run
//! use icarus_dev::{start_dev_server, DevConfig};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let config = DevConfig::new()
//!         .with_watch_paths(vec!["src".to_string()])
//!         .with_auto_reload(true);
//!
//!     start_dev_server(config).await?;
//!     Ok(())
//! }
//! ```
//!
//! ## Modules
//!
//! - [`watch`] - File watching and auto-rebuild functionality
//! - [`status`] - Development environment status checking
//! - [`init`] - Project initialization utilities
//! - [`start`] - Development server startup
//! - [`reset`] - Environment reset tools

pub mod init;
pub mod reset;
pub mod start;
pub mod status;
pub mod utils;
pub mod watch;

// Re-export main types for convenience
// Note: Individual types will be added as they are implemented in their modules

use anyhow::Result;

/// Configuration for development tools
#[derive(Debug, Clone)]
pub struct DevConfig {
    /// Paths to watch for changes
    pub watch_paths: Vec<String>,
    /// Enable automatic reload on changes
    pub auto_reload: bool,
    /// Debounce delay in milliseconds
    pub debounce_delay: u64,
    /// Verbose output
    pub verbose: bool,
}

impl Default for DevConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl DevConfig {
    /// Create a new dev configuration with defaults
    pub fn new() -> Self {
        Self {
            watch_paths: vec!["src".to_string()],
            auto_reload: true,
            debounce_delay: 1000,
            verbose: false,
        }
    }

    /// Set the paths to watch for changes
    pub fn with_watch_paths(mut self, paths: Vec<String>) -> Self {
        self.watch_paths = paths;
        self
    }

    /// Enable or disable automatic reload
    pub fn with_auto_reload(mut self, auto_reload: bool) -> Self {
        self.auto_reload = auto_reload;
        self
    }

    /// Set the debounce delay in milliseconds
    pub fn with_debounce_delay(mut self, delay: u64) -> Self {
        self.debounce_delay = delay;
        self
    }

    /// Enable verbose output
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }
}

/// Start the development server with the given configuration
///
/// This is a convenience function that sets up file watching and
/// development tools based on the provided configuration.
pub async fn start_dev_server(config: DevConfig) -> Result<()> {
    // Use watch instead of start for the file watching functionality
    watch::execute(
        Some(config.watch_paths),
        config.debounce_delay,
        config.verbose,
    )
    .await
}

/// Get the current development status
///
/// This function checks the current state of the development environment
/// and returns detailed status information.
pub async fn get_dev_status(detailed: bool) -> Result<()> {
    status::execute(detailed).await
}

/// Initialize a new development environment
///
/// Sets up the necessary configuration and tools for development.
pub async fn init_dev_environment(skip_checks: bool, force: bool) -> Result<()> {
    init::execute(skip_checks, force).await
}

/// Reset the development environment
///
/// Cleans up development artifacts and resets the environment to a clean state.
pub async fn reset_dev_environment(clean: bool, yes: bool) -> Result<()> {
    reset::execute(clean, yes).await
}
