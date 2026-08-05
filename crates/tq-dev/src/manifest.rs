//! The pinned developer tool manifest at `.github/dev-tools.toml`.

use std::path::{Path, PathBuf};

use crate::error::DevError;
use crate::parse;

pub const DEV_TOOLS_PATH: &str = ".github/dev-tools.toml";
const NODE_TOOLS_PATH: &str = "package.json";

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

    #[must_use]
    pub(crate) const fn major(&self) -> u64 {
        self.major
    }

    #[must_use]
    pub(crate) const fn minor(&self) -> u64 {
        self.minor
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

#[derive(Debug)]
pub(crate) struct NodeToolchain {
    pub(crate) node: ToolVersion,
    pub(crate) npm: ToolVersion,
}

impl NodeToolchain {
    pub(crate) fn load(repo_root: &Path) -> Result<Self, DevError> {
        let path = repo_root.join(NODE_TOOLS_PATH);
        let document = parse::read_json(&path)?;
        let version = |segments: &[&str]| -> Result<ToolVersion, DevError> {
            let text = parse::required_json_string(&document, segments, &path)?;
            ToolVersion::parse(&text).map_err(|message| DevError::InvalidInput {
                path: path.clone(),
                message,
            })
        };

        let package_manager = parse::required_json_string(&document, &["packageManager"], &path)?;
        let npm = package_manager
            .strip_prefix("npm@")
            .ok_or_else(|| DevError::InvalidInput {
                path: path.clone(),
                message: "packageManager must use an exact npm@major.minor.patch version"
                    .to_owned(),
            })?;

        Ok(Self {
            node: version(&["engines", "node"])?,
            npm: ToolVersion::parse(npm)
                .map_err(|message| DevError::InvalidInput { path, message })?,
        })
    }
}

/// Validated contents of `.github/dev-tools.toml`.
#[derive(Debug)]
pub struct DevToolsManifest {
    pub rust: ToolVersion,
    pub python: ToolVersion,
    pub uv: ToolVersion,
    pub maturin: ToolVersion,
    pub actionlint: ToolVersion,
    pub shellcheck: ToolVersion,
    pub(crate) actionlint_image: ContainerImage,
    pub cargo_outdated: ToolVersion,
    pub cargo_audit: ToolVersion,
    pub cargo_deny: ToolVersion,
}

#[derive(Debug)]
pub(crate) struct ContainerImage {
    repository: String,
    digest: String,
}

impl ContainerImage {
    #[must_use]
    pub(crate) fn pinned_reference(&self, version: &ToolVersion) -> String {
        format!(
            "docker://{}:{}@{}",
            self.repository,
            version.as_str(),
            self.digest
        )
    }
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

            maturin: version(&["tools", "maturin"])?,
            actionlint: version(&["tools", "actionlint"])?,
            shellcheck: version(&["tools", "shellcheck"])?,
            actionlint_image: container_image(&document, &path)?,
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

fn container_image(document: &toml::Value, path: &Path) -> Result<ContainerImage, DevError> {
    let repository = parse::required_string(
        document,
        &["automation", "actionlint-image", "repository"],
        path,
    )?;
    if repository.is_empty() || repository.contains([':', '@']) {
        return Err(DevError::InvalidInput {
            path: path.to_path_buf(),
            message: "automation.actionlint-image.repository must be an untagged image repository"
                .to_owned(),
        });
    }

    let digest = parse::required_string(
        document,
        &["automation", "actionlint-image", "digest"],
        path,
    )?;
    let Some(hash) = digest.strip_prefix("sha256:") else {
        return Err(DevError::InvalidInput {
            path: path.to_path_buf(),
            message: "automation.actionlint-image.digest must use sha256".to_owned(),
        });
    };
    if hash.len() != 64
        || !hash
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(DevError::InvalidInput {
            path: path.to_path_buf(),
            message: "automation.actionlint-image.digest must contain 64 lowercase hexadecimal characters"
                .to_owned(),
        });
    }

    Ok(ContainerImage { repository, digest })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::{NodeToolchain, ToolVersion};

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

    #[test]
    fn loads_node_toolchain_from_package_metadata() {
        let temp = tempfile::tempdir().expect("tempdir");
        fs::write(
            temp.path().join("package.json"),
            r#"{"packageManager":"npm@11.17.0","engines":{"node":"26.4.0"}}"#,
        )
        .expect("write package metadata");

        let toolchain = NodeToolchain::load(temp.path()).expect("valid Node toolchain");
        assert_eq!(toolchain.node.as_str(), "26.4.0");
        assert_eq!(toolchain.npm.as_str(), "11.17.0");
    }

    #[test]
    fn rejects_non_exact_npm_package_manager_versions() {
        let temp = tempfile::tempdir().expect("tempdir");
        fs::write(
            temp.path().join("package.json"),
            r#"{"packageManager":"npm@^11.17.0","engines":{"node":"26.4.0"}}"#,
        )
        .expect("write package metadata");

        NodeToolchain::load(temp.path()).expect_err("npm range must fail");
    }
}
