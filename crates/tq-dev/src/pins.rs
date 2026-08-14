//! Dependency and maintenance-tool drift audits.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::audit::{
    AuditCheck, AuditCommand, AuditReport, FindingsSignal, run_audit_command, run_audit_commands,
};
use crate::error::DevError;
use crate::external_pins::{self, ReleaseSeries};
use crate::invocation::Invocation;
use crate::manifest::{CratesIoTool, DevToolsManifest, NodeToolchain, ToolVersion};
use crate::update;

/// Reports available updates for every pinned tool and project dependency ecosystem.
pub fn audit_latest(repo_root: &Path) -> Result<AuditReport, DevError> {
    let manifest = DevToolsManifest::load(repo_root)?;
    let node = NodeToolchain::load(repo_root)?;
    let mut checks = vec![
        rust_toolchain_check(repo_root, &manifest.rust),
        github_release_pin_check(
            repo_root,
            "python",
            "https://github.com/python/cpython.git",
            ReleaseSeries::Minor(manifest.python.major(), manifest.python.minor()),
            &manifest.python,
            "Update the python pin in .github/dev-tools.toml.",
        ),
        github_release_pin_check(
            repo_root,
            "uv",
            "https://github.com/astral-sh/uv.git",
            ReleaseSeries::Any,
            &manifest.uv,
            "Update the uv pin in .github/dev-tools.toml.",
        ),
        github_release_pin_check(
            repo_root,
            "node",
            "https://github.com/nodejs/node.git",
            ReleaseSeries::Major(node.node.major()),
            &node.node,
            "Update engines.node in package.json.",
        ),
        pypi_maturin_pin_check(repo_root, &manifest.maturin).with_remediation(
            "Update the maturin pin in .github/dev-tools.toml and validate the release build.",
        ),
        github_release_pin_check(
            repo_root,
            "actionlint",
            "https://github.com/rhysd/actionlint.git",
            ReleaseSeries::Any,
            &manifest.actionlint,
            "Update the actionlint pin and image digest in .github/dev-tools.toml.",
        ),
        github_release_pin_check(
            repo_root,
            "shellcheck",
            "https://github.com/koalaman/shellcheck.git",
            ReleaseSeries::Any,
            &manifest.shellcheck,
            "Update the shellcheck pin in .github/dev-tools.toml.",
        ),
    ];
    checks.extend(maintenance_tool_checks(repo_root, &manifest)?);
    checks.extend(run_audit_commands(
        repo_root,
        vec![
            AuditCommand {
                name: "cargo-dependencies",
                invocation: Invocation::new("cargo", ["update", "--dry-run"]),
                signal: FindingsSignal::CargoUpdateDryRun,
                remediation: Some("Run cargo update and review Cargo.lock."),
            },
            AuditCommand {
                name: "npm-dependencies",
                invocation: Invocation::new("npm", ["outdated"]),
                signal: FindingsSignal::ExitCodeOne,
                remediation: Some("Run cargo dev deps update and review package-lock.json."),
            },
            AuditCommand {
                name: "python-dependencies",
                invocation: Invocation::new(
                    "uv",
                    ["pip", "list", "--outdated", "--format", "json"],
                ),
                signal: FindingsSignal::OutdatedPackagesJson,
                remediation: Some("Run cargo dev deps update and review uv.lock."),
            },
        ],
    )?);
    Ok(AuditReport::new(checks))
}

/// Runs the pinned security audit tools.
pub fn audit_security(repo_root: &Path) -> Result<AuditReport, DevError> {
    let checks = run_audit_commands(
        repo_root,
        vec![
            AuditCommand {
                name: "cargo-audit",
                invocation: Invocation::new("cargo", ["audit", "-D", "warnings"]),
                signal: FindingsSignal::CargoAudit,
                remediation: Some(
                    "Review every RustSec vulnerability and warning before changing dependency state.",
                ),
            },
            AuditCommand {
                name: "cargo-deny",
                invocation: Invocation::new("cargo", ["deny", "check"]),
                signal: FindingsSignal::CargoDeny,
                remediation: Some(
                    "Review the cargo-deny policy violation before changing dependency state.",
                ),
            },
            AuditCommand {
                name: "uv-audit",
                invocation: Invocation::new("uv", ["audit", "--locked"]),
                signal: FindingsSignal::ExitCodeOne,
                remediation: Some("Update the affected Python dependency and review uv.lock."),
            },
            AuditCommand {
                name: "npm-audit",
                invocation: Invocation::new("npm", ["audit", "--audit-level", "moderate"]),
                signal: FindingsSignal::ExitCodeOne,
                remediation: Some(
                    "Update the affected docs dependency and review package-lock.json.",
                ),
            },
        ],
    )?;
    Ok(AuditReport::new(checks))
}

/// Compares pinned Rust maintenance tools against the latest crates.io
/// releases.
pub fn audit_maintenance_tools(repo_root: &Path) -> Result<AuditReport, DevError> {
    let manifest = DevToolsManifest::load(repo_root)?;
    Ok(AuditReport::new(maintenance_tool_checks(
        repo_root, &manifest,
    )?))
}

fn maintenance_tool_checks(
    repo_root: &Path,
    manifest: &DevToolsManifest,
) -> Result<Vec<AuditCheck>, DevError> {
    let mut checks = Vec::new();
    for (name, tool) in [
        ("cargo-audit", &manifest.cargo_audit),
        ("cargo-deny", &manifest.cargo_deny),
    ] {
        checks.push(crates_io_pin_check(repo_root, name, &tool.version).with_remediation(
            "Update the rust-maintenance pin in .github/dev-tools.toml and its CI setup action defaults.",
        ));
        checks.extend(packaged_tool_checks(repo_root, name, tool)?);
    }
    Ok(checks)
}

fn packaged_tool_checks(
    repo_root: &Path,
    crate_name: &str,
    tool: &CratesIoTool,
) -> Result<Vec<AuditCheck>, DevError> {
    let cargo_home = TemporaryCargoHome::new()?;
    let package = format!("{crate_name}@{}", tool.version);
    let invocation =
        Invocation::with_args("cargo", ["info".to_owned(), package.clone()]).with_env(vec![(
            "CARGO_HOME".to_owned(),
            cargo_home.path().display().to_string(),
        )]);
    let command = invocation.display();
    let captured = invocation.capture(repo_root)?;
    let identity_name = format!("{crate_name}-metadata");
    if !captured.success {
        return Ok(vec![AuditCheck::failed(
            &identity_name,
            &command,
            captured.output,
        )]);
    }

    let Some(repository) = parse_cargo_info_repository(&captured.output) else {
        return Ok(vec![AuditCheck::failed(
            &identity_name,
            &command,
            format!(
                "missing repository metadata in cargo info output\n{}",
                captured.output
            ),
        )]);
    };
    if repository != tool.repository.as_str() {
        return Ok(vec![AuditCheck::failed(
            &identity_name,
            &command,
            format!(
                "expected repository {}, found {repository}",
                tool.repository.as_str()
            ),
        )]);
    }

    let identity = AuditCheck::clean(&identity_name, &command);
    let lockfile = packaged_lockfile(cargo_home.path(), crate_name, &tool.version)?;
    let lock_name = format!("{crate_name}-lock");
    let remediation = format!(
        "Do not install {package} until its published Cargo.lock is free of known advisories."
    );
    let lock_invocation = Invocation::with_args(
        "cargo",
        [
            "audit".to_owned(),
            "--file".to_owned(),
            lockfile.display().to_string(),
            "-D".to_owned(),
            "warnings".to_owned(),
        ],
    );
    let lock = run_audit_command(
        repo_root,
        &lock_name,
        &lock_invocation,
        FindingsSignal::CargoAudit,
        Some(&remediation),
    )?;
    Ok(vec![identity, lock])
}

struct TemporaryCargoHome {
    path: PathBuf,
}

impl TemporaryCargoHome {
    fn new() -> Result<Self, DevError> {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| DevError::InvalidInput {
                path: std::env::temp_dir(),
                message: format!("system clock precedes Unix epoch: {error}"),
            })?
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "tq-maintenance-audit-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir(&path).map_err(|source| DevError::Io {
            path: path.clone(),
            source,
        })?;
        Ok(Self { path })
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TemporaryCargoHome {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn parse_cargo_info_repository(output: &str) -> Option<&str> {
    output
        .lines()
        .find_map(|line| line.trim().strip_prefix("repository: "))
}

fn packaged_lockfile(
    cargo_home: &Path,
    crate_name: &str,
    version: &ToolVersion,
) -> Result<PathBuf, DevError> {
    let source_root = cargo_home.join("registry/src");
    let package_directory = format!("{crate_name}-{version}");
    let mut candidates = Vec::new();
    for registry in fs::read_dir(&source_root).map_err(|source| DevError::Io {
        path: source_root.clone(),
        source,
    })? {
        let registry = registry.map_err(|source| DevError::Io {
            path: source_root.clone(),
            source,
        })?;
        let candidate = registry.path().join(&package_directory).join("Cargo.lock");
        if candidate.is_file() {
            candidates.push(candidate);
        }
    }
    candidates.sort();
    if candidates.len() != 1 {
        return Err(DevError::InvalidInput {
            path: source_root,
            message: format!(
                "expected exactly one packaged Cargo.lock for {crate_name}@{version}, found {}",
                candidates.len()
            ),
        });
    }
    Ok(candidates.remove(0))
}

fn rust_toolchain_check(repo_root: &Path, pinned: &ToolVersion) -> AuditCheck {
    match update::latest_stable_rust(repo_root) {
        Ok(latest) => {
            AuditCheck::pin_comparison("rust", "rustup check", pinned.as_str(), latest.as_str())
                .with_remediation("Run cargo dev deps update to update the repository Rust pins.")
        }
        Err(error) => AuditCheck::failed("rust", "rustup check", error.to_string()),
    }
}

fn github_release_pin_check(
    repo_root: &Path,
    tool: &str,
    remote: &str,
    series: ReleaseSeries,
    pinned: &ToolVersion,
    remediation: &str,
) -> AuditCheck {
    let command = format!("git ls-remote --tags {remote}");
    let release = match external_pins::latest_release_in_series(repo_root, remote, series) {
        Ok(Some(release)) => release,
        Ok(None) => {
            return AuditCheck::failed(
                tool,
                &command,
                format!("no matching SemVer release tags were found for {remote}"),
            );
        }
        Err(message) => return AuditCheck::failed(tool, &command, message),
    };
    let version_text = release.tag.strip_prefix('v').unwrap_or(&release.tag);
    match ToolVersion::parse(version_text) {
        Ok(latest) => AuditCheck::pin_comparison(tool, &command, pinned.as_str(), latest.as_str())
            .with_remediation(remediation),
        Err(message) => AuditCheck::failed(
            tool,
            &command,
            format!(
                "could not parse latest {tool} release {}: {message}",
                release.tag
            ),
        ),
    }
}

fn pypi_maturin_pin_check(repo_root: &Path, pinned: &ToolVersion) -> AuditCheck {
    let invocation = Invocation::new(
        "uv",
        [
            "run",
            "--isolated",
            "--no-project",
            "--with",
            "maturin>=1,<2",
            "--",
            "maturin",
            "--version",
        ],
    );
    let command = invocation.display();
    let captured = match invocation.capture(repo_root) {
        Ok(captured) if captured.success => captured,
        Ok(captured) => return AuditCheck::failed("maturin", &command, captured.output),
        Err(error) => return AuditCheck::failed("maturin", &command, error.to_string()),
    };

    match parse_maturin_version(&captured.output) {
        Some(latest) => {
            AuditCheck::pin_comparison("maturin", &command, pinned.as_str(), latest.as_str())
        }
        None => AuditCheck::failed(
            "maturin",
            &command,
            format!(
                "could not parse latest maturin version from command output\n{}",
                captured.output
            ),
        ),
    }
}

fn parse_maturin_version(output: &str) -> Option<ToolVersion> {
    output.lines().find_map(|line| {
        let version = line.trim().strip_prefix("maturin ")?;
        ToolVersion::parse(version).ok()
    })
}

fn crates_io_pin_check(repo_root: &Path, crate_name: &str, pinned: &ToolVersion) -> AuditCheck {
    let invocation = Invocation::new("cargo", ["search", crate_name, "--limit", "1"]);
    let command = invocation.display();

    let captured = match invocation.capture(repo_root) {
        Ok(captured) => captured,
        Err(error) => return AuditCheck::failed(crate_name, &command, error.to_string()),
    };
    if !captured.success {
        return AuditCheck::failed(crate_name, &command, captured.output);
    }

    match parse_cargo_search_version(crate_name, &captured.output) {
        Some(latest) => AuditCheck::pin_comparison(crate_name, &command, pinned.as_str(), &latest),
        None => AuditCheck::failed(
            crate_name,
            &command,
            format!(
                "could not parse latest {crate_name} version from cargo search output\n{}",
                captured.output
            ),
        ),
    }
}

fn parse_cargo_search_version(crate_name: &str, output: &str) -> Option<String> {
    let prefix = format!("{crate_name} = \"");
    output
        .lines()
        .find_map(|line| line.strip_prefix(&prefix))?
        .split_once('"')
        .map(|(version, _)| version.to_owned())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::{
        packaged_lockfile, parse_cargo_info_repository, parse_cargo_search_version,
        parse_maturin_version,
    };
    use crate::manifest::ToolVersion;

    #[test]
    fn parses_cargo_search_exact_crate_version() {
        assert_eq!(
            parse_cargo_search_version(
                "cargo-deny",
                "cargo-deny = \"0.20.2\"    # Check dependency policy\nother = \"9.9.9\"",
            ),
            Some("0.20.2".to_owned())
        );
    }

    #[test]
    fn parses_cargo_info_repository() {
        assert_eq!(
            parse_cargo_info_repository(
                "cargo-deny #security\nrepository: https://github.com/EmbarkStudios/cargo-deny\n"
            ),
            Some("https://github.com/EmbarkStudios/cargo-deny")
        );
        assert_eq!(
            parse_cargo_info_repository("repository metadata missing"),
            None
        );
    }

    #[test]
    fn finds_one_exact_packaged_lockfile() {
        let temp = tempfile::tempdir().expect("tempdir");
        let lockfile = temp
            .path()
            .join("registry/src/index.crates.io-example/cargo-deny-0.20.2/Cargo.lock");
        fs::create_dir_all(lockfile.parent().expect("lockfile parent"))
            .expect("create package source");
        fs::write(&lockfile, "version = 4\n").expect("write lockfile");
        let version = ToolVersion::parse("0.20.2").expect("valid version");

        assert_eq!(
            packaged_lockfile(temp.path(), "cargo-deny", &version).expect("one exact lockfile"),
            lockfile
        );
    }

    #[test]
    fn parses_maturin_version_output() {
        assert_eq!(
            parse_maturin_version("maturin 1.14.1\n").map(|version| version.to_string()),
            Some("1.14.1".to_owned())
        );
        assert_eq!(
            parse_maturin_version("Installed 1 package in 6ms\nmaturin 1.14.1\n")
                .map(|version| version.to_string()),
            Some("1.14.1".to_owned())
        );
        assert_eq!(parse_maturin_version("maturin 1.14"), None);
    }
}
