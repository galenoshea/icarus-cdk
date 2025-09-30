//! Tool versioning support for semantic version management.
//!
//! This module provides semantic versioning (`SemVer`) support for tools,
//! enabling compatibility checking, version comparison, and upgrade management.

use std::fmt;
use std::str::FromStr;

use candid::{CandidType, Deserialize};
use serde::Serialize;

use crate::IcarusError;

/// Semantic version following the major.minor.patch format.
///
/// This type provides comprehensive version management for tools, enabling
/// compatibility checking, upgrade detection, and version-aware tool selection.
///
/// # Version Components
///
/// - **Major**: Breaking changes that require updates to consumers
/// - **Minor**: Backward-compatible feature additions
/// - **Patch**: Backward-compatible bug fixes and improvements
///
/// # Examples
///
/// ```rust
/// use icarus_core::Version;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Creating versions
/// let v1 = Version::new(1, 2, 3);
/// let v2 = Version::parse("2.0.0")?;
///
/// // Version comparison
/// assert!(v2 > v1);
/// assert!(v1.is_compatible_with(&Version::new(1, 3, 0)));
/// assert!(!v1.is_compatible_with(&Version::new(2, 0, 0)));
///
/// // Display
/// assert_eq!(v1.to_string(), "1.2.3");
/// # Ok(())
/// # }
/// ```
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, CandidType, Serialize, Deserialize,
)]
pub struct Version {
    /// Major version number (breaking changes)
    pub major: u32,
    /// Minor version number (feature additions)
    pub minor: u32,
    /// Patch version number (bug fixes)
    pub patch: u32,
}

impl Version {
    /// Creates a new version with the specified major, minor, and patch numbers.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use icarus_core::Version;
    ///
    /// let version = Version::new(1, 2, 3);
    /// assert_eq!(version.major, 1);
    /// assert_eq!(version.minor, 2);
    /// assert_eq!(version.patch, 3);
    /// ```
    #[must_use]
    #[inline]
    pub const fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    /// Parses a version string in "major.minor.patch" format.
    ///
    /// # Arguments
    ///
    /// * `s` - A string slice containing the version in "X.Y.Z" format
    ///
    /// # Returns
    ///
    /// * `Ok(Version)` if parsing succeeded
    /// * `Err(IcarusError)` if the format is invalid
    ///
    /// # Errors
    ///
    /// Returns `IcarusError::InvalidVersion` if:
    /// - The version string doesn't have exactly 3 parts (major.minor.patch)
    /// - Any part cannot be parsed as a u32
    ///
    /// # Examples
    ///
    /// ```rust
    /// use icarus_core::Version;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let version = Version::parse("1.2.3")?;
    /// assert_eq!(version, Version::new(1, 2, 3));
    ///
    /// // Invalid formats return errors
    /// assert!(Version::parse("1.2").is_err());
    /// assert!(Version::parse("1.2.3.4").is_err());
    /// assert!(Version::parse("v1.2.3").is_err());
    /// # Ok(())
    /// # }
    /// ```
    pub fn parse(s: &str) -> Result<Self, IcarusError> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 3 {
            return Err(IcarusError::InvalidVersion(format!(
                "Version must have exactly 3 parts (major.minor.patch), got: {s}"
            )));
        }

        let major = parts[0].parse::<u32>().map_err(|_| {
            IcarusError::InvalidVersion(format!("Invalid major version: {}", parts[0]))
        })?;

        let minor = parts[1].parse::<u32>().map_err(|_| {
            IcarusError::InvalidVersion(format!("Invalid minor version: {}", parts[1]))
        })?;

        let patch = parts[2].parse::<u32>().map_err(|_| {
            IcarusError::InvalidVersion(format!("Invalid patch version: {}", parts[2]))
        })?;

        Ok(Self::new(major, minor, patch))
    }

    /// Checks if this version is compatible with another version.
    ///
    /// Two versions are compatible if they have the same major version number.
    /// This follows semantic versioning rules where major version changes
    /// indicate breaking changes.
    ///
    /// # Arguments
    ///
    /// * `other` - The version to check compatibility with
    ///
    /// # Returns
    ///
    /// `true` if the versions are compatible (same major version)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use icarus_core::Version;
    ///
    /// let v1_2_3 = Version::new(1, 2, 3);
    /// let v1_3_0 = Version::new(1, 3, 0);
    /// let v2_0_0 = Version::new(2, 0, 0);
    ///
    /// assert!(v1_2_3.is_compatible_with(&v1_3_0));
    /// assert!(!v1_2_3.is_compatible_with(&v2_0_0));
    /// ```
    #[must_use]
    #[inline]
    pub const fn is_compatible_with(&self, other: &Self) -> bool {
        self.major == other.major
    }

    /// Checks if this version can be upgraded to another version.
    ///
    /// A version can be upgraded to another if:
    /// - They are compatible (same major version), AND
    /// - The other version is newer (higher minor/patch)
    ///
    /// # Arguments
    ///
    /// * `other` - The target version to upgrade to
    ///
    /// # Returns
    ///
    /// `true` if upgrade is possible and safe
    ///
    /// # Examples
    ///
    /// ```rust
    /// use icarus_core::Version;
    ///
    /// let v1_2_3 = Version::new(1, 2, 3);
    /// let v1_2_4 = Version::new(1, 2, 4);
    /// let v1_3_0 = Version::new(1, 3, 0);
    /// let v2_0_0 = Version::new(2, 0, 0);
    ///
    /// assert!(v1_2_3.can_upgrade_to(&v1_2_4));
    /// assert!(v1_2_3.can_upgrade_to(&v1_3_0));
    /// assert!(!v1_2_3.can_upgrade_to(&v2_0_0)); // Major version change
    /// assert!(!v1_3_0.can_upgrade_to(&v1_2_3)); // Downgrade
    /// ```
    #[must_use]
    #[inline]
    pub fn can_upgrade_to(&self, other: &Self) -> bool {
        self.is_compatible_with(other) && other > self
    }

    /// Increments the patch version number.
    ///
    /// This creates a new version with the patch number incremented by 1.
    /// Used for backward-compatible bug fixes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use icarus_core::Version;
    ///
    /// let v1 = Version::new(1, 2, 3);
    /// let v2 = v1.bump_patch();
    /// assert_eq!(v2, Version::new(1, 2, 4));
    /// ```
    #[must_use]
    #[inline]
    pub const fn bump_patch(&self) -> Self {
        Self::new(self.major, self.minor, self.patch + 1)
    }

    /// Increments the minor version number and resets patch to 0.
    ///
    /// This creates a new version with the minor number incremented by 1
    /// and patch reset to 0. Used for backward-compatible feature additions.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use icarus_core::Version;
    ///
    /// let v1 = Version::new(1, 2, 3);
    /// let v2 = v1.bump_minor();
    /// assert_eq!(v2, Version::new(1, 3, 0));
    /// ```
    #[must_use]
    #[inline]
    pub const fn bump_minor(&self) -> Self {
        Self::new(self.major, self.minor + 1, 0)
    }

    /// Increments the major version number and resets minor and patch to 0.
    ///
    /// This creates a new version with the major number incremented by 1
    /// and minor/patch reset to 0. Used for breaking changes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use icarus_core::Version;
    ///
    /// let v1 = Version::new(1, 2, 3);
    /// let v2 = v1.bump_major();
    /// assert_eq!(v2, Version::new(2, 0, 0));
    /// ```
    #[must_use]
    #[inline]
    pub const fn bump_major(&self) -> Self {
        Self::new(self.major + 1, 0, 0)
    }

    /// Returns true if this is a pre-release version (0.x.y).
    ///
    /// Pre-release versions have major version 0 and may have different
    /// compatibility semantics.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use icarus_core::Version;
    ///
    /// assert!(Version::new(0, 1, 0).is_prerelease());
    /// assert!(!Version::new(1, 0, 0).is_prerelease());
    /// ```
    #[must_use]
    #[inline]
    pub const fn is_prerelease(&self) -> bool {
        self.major == 0
    }
}

impl Default for Version {
    /// Creates a default version of 1.0.0.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use icarus_core::Version;
    ///
    /// let version = Version::default();
    /// assert_eq!(version, Version::new(1, 0, 0));
    /// ```
    #[inline]
    fn default() -> Self {
        Self::new(1, 0, 0)
    }
}

impl fmt::Display for Version {
    /// Formats the version as "major.minor.patch".
    ///
    /// # Examples
    ///
    /// ```rust
    /// use icarus_core::Version;
    ///
    /// let version = Version::new(1, 2, 3);
    /// assert_eq!(version.to_string(), "1.2.3");
    /// ```
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl FromStr for Version {
    type Err = IcarusError;

    /// Parses a version from a string.
    ///
    /// This is equivalent to `Version::parse()`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use icarus_core::Version;
    /// use std::str::FromStr;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let version = Version::from_str("1.2.3")?;
    /// assert_eq!(version, Version::new(1, 2, 3));
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

/// Version requirement specification for tool dependencies.
///
/// This enum allows specifying version requirements in a flexible way,
/// enabling precise control over tool compatibility and upgrades.
///
/// # Examples
///
/// ```rust
/// use icarus_core::{Version, VersionReq};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let version = Version::new(1, 2, 3);
///
/// // Exact version match
/// assert!(VersionReq::Exact(version).matches(&version));
///
/// // Compatible version (same major)
/// let compatible = VersionReq::Compatible(Version::new(1, 0, 0));
/// assert!(compatible.matches(&Version::new(1, 2, 3)));
/// assert!(!compatible.matches(&Version::new(2, 0, 0)));
///
/// // Minimum version
/// let minimum = VersionReq::AtLeast(Version::new(1, 2, 0));
/// assert!(minimum.matches(&Version::new(1, 2, 3)));
/// assert!(!minimum.matches(&Version::new(1, 1, 9)));
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum VersionReq {
    /// Exact version match required
    Exact(Version),
    /// Compatible version (same major, any minor/patch >= specified)
    Compatible(Version),
    /// Minimum version (any version >= specified)
    AtLeast(Version),
    /// Any version is acceptable
    Any,
}

impl VersionReq {
    /// Checks if a version satisfies this requirement.
    ///
    /// # Arguments
    ///
    /// * `version` - The version to check against this requirement
    ///
    /// # Returns
    ///
    /// `true` if the version satisfies the requirement
    ///
    /// # Examples
    ///
    /// ```rust
    /// use icarus_core::{Version, VersionReq};
    ///
    /// let version = Version::new(1, 2, 3);
    ///
    /// assert!(VersionReq::Exact(version).matches(&version));
    /// assert!(VersionReq::Compatible(Version::new(1, 0, 0)).matches(&version));
    /// assert!(VersionReq::AtLeast(Version::new(1, 2, 0)).matches(&version));
    /// assert!(VersionReq::Any.matches(&version));
    /// ```
    #[must_use]
    pub fn matches(&self, version: &Version) -> bool {
        match self {
            Self::Exact(required) => version == required,
            Self::Compatible(required) => {
                version.is_compatible_with(required) && version >= required
            }
            Self::AtLeast(required) => version >= required,
            Self::Any => true,
        }
    }

    /// Parses a version requirement from a string.
    ///
    /// Supported formats:
    /// - "1.2.3" - Exact version
    /// - "^1.2.3" - Compatible version (same major, >= minor.patch)
    /// - ">=1.2.3" - At least version
    /// - "*" - Any version
    ///
    /// # Errors
    ///
    /// Returns `IcarusError::InvalidVersion` if:
    /// - The version requirement format is not recognized
    /// - The version string within the requirement is invalid
    ///
    /// # Examples
    ///
    /// ```rust
    /// use icarus_core::{Version, VersionReq};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let exact = VersionReq::parse("1.2.3")?;
    /// let compatible = VersionReq::parse("^1.2.3")?;
    /// let at_least = VersionReq::parse(">=1.2.3")?;
    /// let any = VersionReq::parse("*")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn parse(s: &str) -> Result<Self, IcarusError> {
        if s == "*" {
            return Ok(Self::Any);
        }

        if let Some(version_str) = s.strip_prefix(">=") {
            let version = Version::parse(version_str)?;
            return Ok(Self::AtLeast(version));
        }

        if let Some(version_str) = s.strip_prefix("^") {
            let version = Version::parse(version_str)?;
            return Ok(Self::Compatible(version));
        }

        // Default to exact match
        let version = Version::parse(s)?;
        Ok(Self::Exact(version))
    }
}

impl Default for VersionReq {
    /// Creates a default version requirement that accepts any version.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use icarus_core::{Version, VersionReq};
    ///
    /// let req = VersionReq::default();
    /// assert!(req.matches(&Version::new(1, 2, 3)));
    /// assert!(req.matches(&Version::new(0, 1, 0)));
    /// ```
    #[inline]
    fn default() -> Self {
        Self::Any
    }
}

impl fmt::Display for VersionReq {
    /// Formats the version requirement in a human-readable format.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use icarus_core::{Version, VersionReq};
    ///
    /// assert_eq!(VersionReq::Exact(Version::new(1, 2, 3)).to_string(), "1.2.3");
    /// assert_eq!(VersionReq::Compatible(Version::new(1, 2, 3)).to_string(), "^1.2.3");
    /// assert_eq!(VersionReq::AtLeast(Version::new(1, 2, 3)).to_string(), ">=1.2.3");
    /// assert_eq!(VersionReq::Any.to_string(), "*");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Exact(version) => write!(f, "{version}"),
            Self::Compatible(version) => write!(f, "^{version}"),
            Self::AtLeast(version) => write!(f, ">={version}"),
            Self::Any => write!(f, "*"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_creation() {
        let version = Version::new(1, 2, 3);
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
    }

    #[test]
    fn test_version_parsing() {
        let version = Version::parse("1.2.3").unwrap();
        assert_eq!(version, Version::new(1, 2, 3));

        assert!(Version::parse("1.2").is_err());
        assert!(Version::parse("1.2.3.4").is_err());
        assert!(Version::parse("v1.2.3").is_err());
        assert!(Version::parse("1.2.x").is_err());
    }

    #[test]
    fn test_version_compatibility() {
        let v1_2_3 = Version::new(1, 2, 3);
        let v1_3_0 = Version::new(1, 3, 0);
        let v2_0_0 = Version::new(2, 0, 0);

        assert!(v1_2_3.is_compatible_with(&v1_3_0));
        assert!(!v1_2_3.is_compatible_with(&v2_0_0));
    }

    #[test]
    fn test_version_upgrades() {
        let v1_2_3 = Version::new(1, 2, 3);
        let v1_2_4 = Version::new(1, 2, 4);
        let v1_3_0 = Version::new(1, 3, 0);
        let v2_0_0 = Version::new(2, 0, 0);

        assert!(v1_2_3.can_upgrade_to(&v1_2_4));
        assert!(v1_2_3.can_upgrade_to(&v1_3_0));
        assert!(!v1_2_3.can_upgrade_to(&v2_0_0));
        assert!(!v1_3_0.can_upgrade_to(&v1_2_3));
    }

    #[test]
    fn test_version_bumping() {
        let version = Version::new(1, 2, 3);

        assert_eq!(version.bump_patch(), Version::new(1, 2, 4));
        assert_eq!(version.bump_minor(), Version::new(1, 3, 0));
        assert_eq!(version.bump_major(), Version::new(2, 0, 0));
    }

    #[test]
    fn test_version_display() {
        let version = Version::new(1, 2, 3);
        assert_eq!(version.to_string(), "1.2.3");
    }

    #[test]
    fn test_version_ordering() {
        let v1_2_3 = Version::new(1, 2, 3);
        let v1_2_4 = Version::new(1, 2, 4);
        let v1_3_0 = Version::new(1, 3, 0);
        let v2_0_0 = Version::new(2, 0, 0);

        assert!(v1_2_3 < v1_2_4);
        assert!(v1_2_3 < v1_3_0);
        assert!(v1_2_3 < v2_0_0);
        assert!(v1_2_4 < v1_3_0);
        assert!(v1_3_0 < v2_0_0);
    }

    #[test]
    fn test_version_requirements() {
        let version = Version::new(1, 2, 3);

        assert!(VersionReq::Exact(version).matches(&version));
        assert!(!VersionReq::Exact(version).matches(&Version::new(1, 2, 4)));

        assert!(VersionReq::Compatible(Version::new(1, 0, 0)).matches(&version));
        assert!(!VersionReq::Compatible(Version::new(2, 0, 0)).matches(&version));

        assert!(VersionReq::AtLeast(Version::new(1, 2, 0)).matches(&version));
        assert!(!VersionReq::AtLeast(Version::new(1, 3, 0)).matches(&version));

        assert!(VersionReq::Any.matches(&version));
    }

    #[test]
    fn test_version_req_parsing() {
        assert_eq!(
            VersionReq::parse("1.2.3").unwrap(),
            VersionReq::Exact(Version::new(1, 2, 3))
        );
        assert_eq!(
            VersionReq::parse("^1.2.3").unwrap(),
            VersionReq::Compatible(Version::new(1, 2, 3))
        );
        assert_eq!(
            VersionReq::parse(">=1.2.3").unwrap(),
            VersionReq::AtLeast(Version::new(1, 2, 3))
        );
        assert_eq!(VersionReq::parse("*").unwrap(), VersionReq::Any);
    }

    #[test]
    fn test_prerelease_versions() {
        assert!(Version::new(0, 1, 0).is_prerelease());
        assert!(!Version::new(1, 0, 0).is_prerelease());
    }
}
