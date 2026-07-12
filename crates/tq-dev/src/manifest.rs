//! The pinned developer tool manifest at `.github/dev-tools.toml`.

use std::path::{Path, PathBuf};

use crate::error::DevError;
use crate::parse;

pub const DEV_TOOLS_PATH: &str = ".github/dev-tools.toml";

const SUPPORTED_SCHEMA_VERSION: i64 = 1;

/// A pinned `major.minor.patch` tool version, validated at the manifest
/// boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolVersion {
    text: String,
    major: u64,
    minor: u64,
    patch: u64,
}

impl ToolVersion {
    pub fn parse(text: &str) -> Result<Self, String> {
        let mut parts = text.split('.');
        let (Some(major), Some(minor), Some(patch), None) =
            (parts.next(), parts.next(), parts.next(), parts.next())
        else {
            return Err(format!(
                "tool version {text:?} must have exactly three dot-separated components"
            ));
        };
        let parse_component = |component: &str| {
            component.parse::<u64>().map_err(|_| {
                format!("tool version {text:?} component {component:?} must be a number")
            })
        };
        Ok(Self {
            text: text.to_owned(),
            major: parse_component(major)?,
            minor: parse_component(minor)?,
            patch: parse_component(patch)?,
        })
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.text
    }

    /// The `major.minor` prefix, as used for the workspace MSRV pin.
    #[must_use]
    pub fn minor_pin(&self) -> String {
        format!("{}.{}", self.major, self.minor)
    }

    /// Whether `candidate` is exactly this version, ignoring surrounding text
    /// such as a program name in `--version` output.
    #[must_use]
    pub fn matches_version_output(&self, output: &str) -> bool {
        output
            .split(|character: char| !(character.is_ascii_digit() || character == '.'))
            .any(|token| token == self.text)
    }
}

impl std::fmt::Display for ToolVersion {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.text)
    }
}

/// Validated contents of `.github/dev-tools.toml`.
#[derive(Debug)]
pub struct DevToolsManifest {
    pub rust: ToolVersion,
    pub python: ToolVersion,
    pub uv: ToolVersion,
    pub node: ToolVersion,
    pub npm: ToolVersion,
    pub mise: ToolVersion,
    pub maturin: ToolVersion,
    pub cargo_outdated: ToolVersion,
    pub cargo_audit: ToolVersion,
    pub cargo_deny: ToolVersion,
}

impl DevToolsManifest {
    pub fn load(repo_root: &Path) -> Result<Self, DevError> {
        let path = repo_root.join(DEV_TOOLS_PATH);
        let document = parse::read_toml(&path)?;

        let schema_version = parse::required_integer(&document, &["schema", "version"], &path)?;
        if schema_version != SUPPORTED_SCHEMA_VERSION {
            return Err(DevError::InvalidInput {
                path,
                message: format!("unsupported dev-tools schema version {schema_version}"),
            });
        }

        let version = |segments: &[&str]| -> Result<ToolVersion, DevError> {
            let text = parse::required_string(&document, segments, &path)?;
            ToolVersion::parse(&text).map_err(|message| DevError::InvalidInput {
                path: path.clone(),
                message,
            })
        };

        Ok(Self {
            rust: version(&["tools", "rust"])?,
            python: version(&["tools", "python"])?,
            uv: version(&["tools", "uv"])?,
            node: version(&["tools", "node"])?,
            npm: version(&["tools", "npm"])?,
            mise: version(&["tools", "mise"])?,
            maturin: version(&["tools", "maturin"])?,
            cargo_outdated: version(&["rust-maintenance", "cargo-outdated"])?,
            cargo_audit: version(&["rust-maintenance", "cargo-audit"])?,
            cargo_deny: version(&["rust-maintenance", "cargo-deny"])?,
        })
    }

    #[must_use]
    pub fn path(repo_root: &Path) -> PathBuf {
        repo_root.join(DEV_TOOLS_PATH)
    }
}

#[cfg(test)]
mod tests {
    use super::ToolVersion;

    #[test]
    fn parses_three_part_versions_only() {
        let version = ToolVersion::parse("1.96.1").expect("valid version");
        assert_eq!(version.as_str(), "1.96.1");
        assert_eq!(version.minor_pin(), "1.96");

        assert!(ToolVersion::parse("1.96").is_err());
        assert!(ToolVersion::parse("1.96.1.0").is_err());
        assert!(ToolVersion::parse("1.96.x").is_err());
    }

    #[test]
    fn version_output_matching_is_exact_not_substring() {
        let version = ToolVersion::parse("0.19.0").expect("valid version");
        assert!(version.matches_version_output("cargo-deny 0.19.0"));
        assert!(!version.matches_version_output("cargo-deny 0.19.01"));
        assert!(!version.matches_version_output("cargo-deny 10.19.0"));

        let rust = ToolVersion::parse("1.96.1").expect("valid version");
        assert!(rust.matches_version_output("rustc 1.96.1 (abcdef 2026-01-01)"));
        assert!(!rust.matches_version_output("rustc 1.96.10"));
    }
}
