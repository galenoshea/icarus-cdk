# Icarus CLI Installation Guide

## Quick Install (Recommended)

The easiest way to install the Icarus CLI is using our installation script:

```bash
curl -L https://icarus.dev/install.sh | sh
```

This will:
1. Detect your operating system and architecture
2. Download the appropriate binary
3. Install it to `/usr/local/bin/icarus`
4. Make it available in your PATH

## System Requirements

- **Operating Systems**: macOS, Linux, Windows (via WSL)
- **Architecture**: x64, ARM64 (Apple Silicon)
- **Dependencies**: 
  - Internet connection for downloading
  - `curl` or `wget` for the install script
  - dfx (installed automatically if missing)

## Platform-Specific Installation

### macOS

#### Intel Macs:
```bash
curl -L https://icarus.dev/downloads/cli/latest/darwin-x64 -o icarus
chmod +x icarus
sudo mv icarus /usr/local/bin/
```

#### Apple Silicon (M1/M2):
```bash
curl -L https://icarus.dev/downloads/cli/latest/darwin-arm64 -o icarus
chmod +x icarus
sudo mv icarus /usr/local/bin/
```

### Linux

#### x64:
```bash
curl -L https://icarus.dev/downloads/cli/latest/linux-x64 -o icarus
chmod +x icarus
sudo mv icarus /usr/local/bin/
```

#### ARM64:
```bash
curl -L https://icarus.dev/downloads/cli/latest/linux-arm64 -o icarus
chmod +x icarus
sudo mv icarus /usr/local/bin/
```

### Windows

Use Windows Subsystem for Linux (WSL) and follow the Linux instructions above.

## Building from Source

If you prefer to build from source:

### Prerequisites
- Rust 1.75 or later
- Git

### Steps

1. Clone the repository:
```bash
git clone https://github.com/icarus-mcp/icarus-cli
cd icarus-cli
```

2. Build the project:
```bash
cargo build --release
```

3. Install the binary:
```bash
cargo install --path .
```

Or manually:
```bash
sudo cp target/release/icarus /usr/local/bin/
```

## Verify Installation

After installation, verify it works:

```bash
icarus --version
```

You should see output like:
```
icarus 0.1.0
```

Test the help command:
```bash
icarus --help
```

## Post-Installation Setup

### 1. Install dfx (if not already installed)

The Icarus CLI uses dfx for ICP interactions. If you don't have it:

```bash
sh -ci "$(curl -fsSL https://internetcomputer.org/install.sh)"
```

### 2. Configure Shell Completion (Optional)

For bash:
```bash
icarus completions bash > ~/.local/share/bash-completion/completions/icarus
```

For zsh:
```bash
icarus completions zsh > ~/.zfunc/_icarus
```

For fish:
```bash
icarus completions fish > ~/.config/fish/completions/icarus.fish
```

### 3. Set Up Configuration Directory

The CLI stores configuration in `~/.icarus/`. This is created automatically on first use.

## Updating

The CLI includes a self-update mechanism:

```bash
icarus update
```

This will:
1. Check for newer versions
2. Download the update if available
3. Replace the current binary
4. Preserve your configuration

To check for updates without installing:
```bash
icarus update --check
```

## Uninstalling

To remove the Icarus CLI:

1. Remove the binary:
```bash
sudo rm /usr/local/bin/icarus
```

2. Remove configuration (optional):
```bash
rm -rf ~/.icarus
```

3. Remove shell completions (if installed):
```bash
# Bash
rm ~/.local/share/bash-completion/completions/icarus

# Zsh
rm ~/.zfunc/_icarus

# Fish
rm ~/.config/fish/completions/icarus.fish
```

## Troubleshooting

### Permission Denied

If you get "permission denied" errors:
```bash
# Make sure the binary is executable
chmod +x icarus

# Use sudo for system directories
sudo mv icarus /usr/local/bin/
```

### Command Not Found

If `icarus` is not found after installation:

1. Check if it's in your PATH:
```bash
echo $PATH
```

2. Add to PATH if needed:
```bash
# Add to ~/.bashrc or ~/.zshrc
export PATH="$PATH:/usr/local/bin"
```

3. Reload your shell:
```bash
source ~/.bashrc  # or ~/.zshrc
```

### SSL/TLS Errors

If you get SSL errors during download:
```bash
# Use -k flag to skip certificate verification (not recommended)
curl -Lk https://icarus.dev/install.sh | sh
```

Better solution: Update your system's certificates:
```bash
# macOS
brew install ca-certificates

# Linux (Debian/Ubuntu)
sudo apt-get update && sudo apt-get install ca-certificates

# Linux (RHEL/CentOS)
sudo yum install ca-certificates
```

### Behind a Proxy

If you're behind a corporate proxy:

1. Set proxy environment variables:
```bash
export HTTP_PROXY=http://proxy.company.com:8080
export HTTPS_PROXY=http://proxy.company.com:8080
```

2. Run the installation:
```bash
curl -L https://icarus.dev/install.sh | sh
```

## Getting Help

If you encounter issues:

1. Check the [troubleshooting guide](troubleshooting.md)
2. Visit [GitHub Issues](https://github.com/icarus-mcp/icarus-cli/issues)
3. Join our community Discord
4. Email support@icarus.dev