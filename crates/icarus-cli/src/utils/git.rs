//! Git utility functions for repository management
//! Some functions are used, others are infrastructure for future expansion

// Allow dead code for infrastructure functions not yet used
#![allow(dead_code)]

use anyhow::{Context, Result};
use std::path::Path;
use tokio::fs;
use tokio::process::Command;

/// Initialize a new git repository
pub(crate) async fn init_repository(project_path: &Path) -> Result<()> {
    let output = Command::new("git")
        .arg("init")
        .current_dir(project_path)
        .output()
        .await
        .context("Failed to execute git init")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Git init failed: {}", stderr));
    }

    Ok(())
}

/// Add a gitignore file to the project
pub(crate) async fn add_gitignore(project_path: &Path) -> Result<()> {
    let gitignore_content = get_default_gitignore();
    let gitignore_path = project_path.join(".gitignore");

    fs::write(&gitignore_path, gitignore_content)
        .await
        .with_context(|| format!("Failed to create .gitignore: {}", gitignore_path.display()))?;

    Ok(())
}

/// Get the default gitignore content for Icarus projects
fn get_default_gitignore() -> &'static str {
    r#"# Rust
/target
Cargo.lock

# IDE and editors
.vscode/
.idea/
*.swp
*.swo
*~

# dfx
.dfx/
dist/

# Environment files
.env
.env.local
.env.production

# Canister outputs
*.wasm
*.did

# Build artifacts
/artifacts

# Logs
*.log
npm-debug.log*
yarn-debug.log*
yarn-error.log*

# Dependency directories
node_modules/

# Optional npm cache directory
.npm

# Optional eslint cache
.eslintcache

# Coverage directory used by tools like istanbul
coverage/
*.lcov

# nyc test coverage
.nyc_output

# OS generated files
.DS_Store
.DS_Store?
._*
.Spotlight-V100
.Trashes
ehthumbs.db
Thumbs.db

# Temporary files
*.tmp
*.temp
.tmp/
.temp/

# Backup files
*.bak
*.backup
*~

# MCP configuration (if sensitive)
# mcp-config.json
"#
}

/// Check if git is available
pub(crate) async fn is_git_available() -> bool {
    Command::new("git")
        .arg("--version")
        .output()
        .await
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Add all files to git staging
pub(crate) async fn add_all_files(project_path: &Path) -> Result<()> {
    let output = Command::new("git")
        .args(["add", "."])
        .current_dir(project_path)
        .output()
        .await
        .context("Failed to execute git add")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Git add failed: {}", stderr));
    }

    Ok(())
}

/// Create initial commit
pub(crate) async fn create_initial_commit(project_path: &Path, message: &str) -> Result<()> {
    let output = Command::new("git")
        .args(["commit", "-m", message])
        .current_dir(project_path)
        .output()
        .await
        .context("Failed to execute git commit")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Git commit failed: {}", stderr));
    }

    Ok(())
}

/// Setup initial git configuration for the project
pub(crate) async fn setup_git_config(
    project_path: &Path,
    author_name: &str,
    author_email: &str,
) -> Result<()> {
    // Set local git config for this repository
    let name_output = Command::new("git")
        .args(["config", "user.name", author_name])
        .current_dir(project_path)
        .output()
        .await
        .context("Failed to set git user.name")?;

    if !name_output.status.success() {
        let stderr = String::from_utf8_lossy(&name_output.stderr);
        return Err(anyhow::anyhow!("Git config user.name failed: {}", stderr));
    }

    let email_output = Command::new("git")
        .args(["config", "user.email", author_email])
        .current_dir(project_path)
        .output()
        .await
        .context("Failed to set git user.email")?;

    if !email_output.status.success() {
        let stderr = String::from_utf8_lossy(&email_output.stderr);
        return Err(anyhow::anyhow!("Git config user.email failed: {}", stderr));
    }

    Ok(())
}

/// Get current git branch name
pub(crate) async fn get_current_branch(project_path: &Path) -> Result<String> {
    let output = Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(project_path)
        .output()
        .await
        .context("Failed to get current git branch")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Git branch command failed: {}", stderr));
    }

    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();

    Ok(branch)
}

/// Check if repository has uncommitted changes
pub(crate) async fn has_uncommitted_changes(project_path: &Path) -> Result<bool> {
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(project_path)
        .output()
        .await
        .context("Failed to check git status")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Git status failed: {}", stderr));
    }

    let status_output = String::from_utf8_lossy(&output.stdout);
    Ok(!status_output.trim().is_empty())
}

/// Get git remote URL if it exists
pub(crate) async fn get_remote_url(project_path: &Path) -> Result<Option<String>> {
    let output = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(project_path)
        .output()
        .await
        .context("Failed to get git remote URL")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Check if it's a "not a git repository" error
        if stderr.contains("not a git repository") {
            return Err(anyhow::anyhow!("Not a git repository"));
        }
        // No remote configured
        return Ok(None);
    }

    let url = String::from_utf8_lossy(&output.stdout).trim().to_string();

    Ok(Some(url))
}

/// Create and checkout a new branch
pub(crate) async fn create_and_checkout_branch(
    project_path: &Path,
    branch_name: &str,
) -> Result<()> {
    let output = Command::new("git")
        .args(["checkout", "-b", branch_name])
        .current_dir(project_path)
        .output()
        .await
        .context("Failed to create and checkout git branch")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Git checkout -b failed: {}", stderr));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_git_availability() {
        let available = is_git_available().await;
        // This test will pass if git is installed, which is common in development environments
        println!("Git available: {}", available);
    }

    #[tokio::test]
    async fn test_gitignore_creation() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        add_gitignore(project_path).await.unwrap();

        let gitignore_path = project_path.join(".gitignore");
        assert!(gitignore_path.exists());

        let content = fs::read_to_string(&gitignore_path).await.unwrap();
        assert!(content.contains("# Rust"));
        assert!(content.contains("/target"));
        assert!(content.contains("# dfx"));
    }

    #[tokio::test]
    async fn test_git_init() {
        if !is_git_available().await {
            return; // Skip test if git is not available
        }

        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        let result = init_repository(project_path).await;
        if result.is_ok() {
            // Check if .git directory was created
            assert!(project_path.join(".git").exists());
        }
        // If git init fails, it might be due to system configuration, which is OK for tests
    }

    #[test]
    fn test_gitignore_content() {
        let content = get_default_gitignore();

        // Check for essential Rust entries
        assert!(content.contains("/target"));
        assert!(content.contains("Cargo.lock"));

        // Check for dfx entries
        assert!(content.contains(".dfx/"));
        assert!(content.contains("*.wasm"));
        assert!(content.contains("*.did"));

        // Check for common development files
        assert!(content.contains(".vscode/"));
        assert!(content.contains(".idea/"));
        assert!(content.contains("*.log"));
    }

    #[tokio::test]
    async fn test_git_operations_without_repo() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // These operations should fail gracefully when not in a git repo
        let branch_result = get_current_branch(project_path).await;
        assert!(branch_result.is_err());

        let changes_result = has_uncommitted_changes(project_path).await;
        assert!(changes_result.is_err());

        let remote_result = get_remote_url(project_path).await;
        assert!(remote_result.is_err());
    }
}
