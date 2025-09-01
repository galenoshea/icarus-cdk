use anyhow::Result;
use os_info::Type as OsType;
use std::env;

#[derive(Debug, Clone, PartialEq)]
pub enum Platform {
    MacOsX64,
    MacOsArm64,
    LinuxX64,
    LinuxArm64,
    WindowsX64,
}

impl Platform {
    pub fn detect() -> Result<Self> {
        let info = os_info::get();
        let arch = env::consts::ARCH;

        match (info.os_type(), arch) {
            (OsType::Macos, "x86_64") => Ok(Platform::MacOsX64),
            (OsType::Macos, "aarch64") => Ok(Platform::MacOsArm64),
            (OsType::Linux, "x86_64") => Ok(Platform::LinuxX64),
            (OsType::Linux, "aarch64") => Ok(Platform::LinuxArm64),
            (OsType::Windows, "x86_64") => Ok(Platform::WindowsX64),
            _ => anyhow::bail!("Unsupported platform: {} {}", info.os_type(), arch),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Platform::MacOsX64 => "darwin-x64",
            Platform::MacOsArm64 => "darwin-arm64",
            Platform::LinuxX64 => "linux-x64",
            Platform::LinuxArm64 => "linux-arm64",
            Platform::WindowsX64 => "windows-x64",
        }
    }

    pub fn bridge_binary_name(&self) -> &'static str {
        match self {
            Platform::WindowsX64 => "icarus-bridge.exe",
            _ => "icarus-bridge",
        }
    }

    pub fn archive_extension(&self) -> &'static str {
        match self {
            Platform::WindowsX64 => ".zip",
            _ => ".tar.gz",
        }
    }
}

pub fn get_bridge_install_path() -> Result<std::path::PathBuf> {
    let config_dir = crate::config::IcarusConfig::config_dir()?;
    Ok(config_dir.join("bin"))
}

pub fn get_bridge_binary_path() -> Result<std::path::PathBuf> {
    let platform = Platform::detect()?;
    let install_path = get_bridge_install_path()?;
    Ok(install_path.join(platform.bridge_binary_name()))
}

pub fn get_bridge_download_url(version: Option<&str>) -> Result<String> {
    let platform = Platform::detect()?;
    let version = version.unwrap_or("latest");

    Ok(format!(
        "{}/{}/icarus-bridge-{}{}",
        crate::config::BRIDGE_DOWNLOAD_BASE_URL,
        version,
        platform.as_str(),
        platform.archive_extension()
    ))
}
