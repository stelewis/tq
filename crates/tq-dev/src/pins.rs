//! Dependency and maintenance-tool drift audits.

use std::path::Path;

use crate::audit::{AuditCheck, AuditCommand, AuditReport, FindingsSignal, run_audit_commands};
use crate::error::DevError;
use crate::external_pins::{self, ReleaseSeries};
use crate::invocation::Invocation;
use crate::manifest::{DevToolsManifest, NodeToolchain, ToolVersion};
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
    checks.extend(maintenance_tool_checks(repo_root, &manifest));
    checks.extend(run_audit_commands(
        repo_root,
        vec![
            AuditCommand {
                name: "cargo-dependencies",
                invocation: Invocation::new(
                    "cargo",
                    [
                        "+stable",
                        "outdated",
                        "--workspace",
                        "--root-deps-only",
                        "--exit-code",
                        "1",
                    ],
                ),
                signal: FindingsSignal::ExitCodeOne,
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
                invocation: Invocation::new("cargo", ["audit"]),
                signal: FindingsSignal::ExitCodeOne,
                remediation: Some(
                    "Review the advisory and update or explicitly deny the affected dependency.",
                ),
            },
            AuditCommand {
                name: "cargo-deny",
                invocation: Invocation::new("cargo", ["deny", "check"]),
                signal: FindingsSignal::ExitCodeOne,
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
    )))
}

fn maintenance_tool_checks(repo_root: &Path, manifest: &DevToolsManifest) -> Vec<AuditCheck> {
    [
        ("cargo-outdated", &manifest.cargo_outdated),
        ("cargo-audit", &manifest.cargo_audit),
        ("cargo-deny", &manifest.cargo_deny),
    ]
    .into_iter()
    .map(|(name, pinned)| {
        crates_io_pin_check(repo_root, name, pinned).with_remediation(
            "Update the rust-maintenance pin in .github/dev-tools.toml and its CI setup action defaults.",
        )
    })
    .collect()
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
    use super::{parse_cargo_search_version, parse_maturin_version};

    #[test]
    fn parses_cargo_search_exact_crate_version() {
        assert_eq!(
            parse_cargo_search_version(
                "cargo-audit",
                "cargo-audit = \"0.22.2\"    # Audit Cargo.lock\nother = \"9.9.9\"",
            ),
            Some("0.22.2".to_owned())
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
