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
            "Update the Python pin and CI setup action, then update the interpreter with its owning Python manager.",
        ),
        github_release_pin_check(
            repo_root,
            "uv",
            "https://github.com/astral-sh/uv.git",
            ReleaseSeries::Any,
            &manifest.uv,
            "Update the uv pin and CI setup action, then update uv with the package manager that installed it.",
        ),
        github_release_pin_check(
            repo_root,
            "node",
            "https://github.com/nodejs/node.git",
            ReleaseSeries::Major(node.node.major()),
            &node.node,
            "Update package.json and package-lock.json, then update Node with its owning environment manager.",
        ),
        npm_registry_pin_check(repo_root, &node.npm),
        crates_io_pin_check(repo_root, "maturin", &manifest.maturin).with_remediation(
            "Update the maturin pin in .github/dev-tools.toml and validate the release build.",
        ),
        github_release_pin_check(
            repo_root,
            "actionlint",
            "https://github.com/rhysd/actionlint.git",
            ReleaseSeries::Any,
            &manifest.actionlint,
            "Update the actionlint pin and official image digest, then update the local executable with its owning environment manager.",
        ),
        github_release_pin_check(
            repo_root,
            "shellcheck",
            "https://github.com/koalaman/shellcheck.git",
            ReleaseSeries::Any,
            &manifest.shellcheck,
            "Update the ShellCheck pin, then update the local executable with its owning environment manager.",
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
                remediation: Some(
                    "Run cargo dev deps update, review the lockfile and release impact, then update installed tooling through its owner if required.",
                ),
            },
            AuditCommand {
                name: "npm-dependencies",
                invocation: Invocation::new("npm", ["outdated"]),
                signal: FindingsSignal::ExitCodeOne,
                remediation: Some(
                    "Run cargo dev deps update and review package-lock.json and the docs build.",
                ),
            },
            AuditCommand {
                name: "python-dependencies",
                invocation: Invocation::new(
                    "uv",
                    ["pip", "list", "--outdated", "--format", "json"],
                ),
                signal: FindingsSignal::JsonArrayNonEmpty,
                remediation: Some(
                    "Run cargo dev deps update and review uv.lock and the Python tooling checks.",
                ),
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
            "Update the maintenance-tool pin in .github/dev-tools.toml and its CI setup action; update the local executable with Cargo or its owning environment manager.",
        )
    })
    .collect()
}

fn rust_toolchain_check(repo_root: &Path, pinned: &ToolVersion) -> AuditCheck {
    match update::latest_stable_rust(repo_root) {
        Ok(latest) => AuditCheck::pin_comparison(
            "rust",
            "rustup check",
            pinned.as_str(),
            latest.as_str(),
        )
        .with_remediation(
            "Run cargo dev deps update to update repository Rust pins, then update the selected toolchain with rustup or its owning environment manager.",
        ),
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

fn npm_registry_pin_check(repo_root: &Path, pinned: &ToolVersion) -> AuditCheck {
    let requirement = format!("npm@{}", pinned.major());
    let invocation = Invocation::new("npm", ["view", &requirement, "version", "--json"]);
    let command = invocation.display();
    let captured = match invocation.capture(repo_root) {
        Ok(captured) if captured.success => captured,
        Ok(captured) => return AuditCheck::failed("npm", &command, captured.output),
        Err(error) => return AuditCheck::failed("npm", &command, error.to_string()),
    };

    match parse_npm_versions(&captured.output) {
        Some(latest) => AuditCheck::pin_comparison("npm", &command, pinned.as_str(), &latest)
            .with_remediation(
                "Update package.json and package-lock.json, then update npm with the Node toolchain manager that owns it.",
            ),
        None => AuditCheck::failed(
            "npm",
            &command,
            format!("could not parse npm registry versions from {}", captured.output),
        ),
    }
}

fn parse_npm_versions(output: &str) -> Option<String> {
    let value = serde_json::from_str::<serde_json::Value>(output).ok()?;
    let values = match value {
        serde_json::Value::String(version) => vec![version],
        serde_json::Value::Array(values) => values
            .into_iter()
            .map(|value| value.as_str().map(ToOwned::to_owned))
            .collect::<Option<Vec<_>>>()?,
        _ => return None,
    };
    values
        .into_iter()
        .filter_map(|value| ToolVersion::parse(&value).ok())
        .max_by_key(|version| (version.major(), version.minor(), version.patch()))
        .map(|version| version.as_str().to_owned())
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
    use super::{parse_cargo_search_version, parse_npm_versions};

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
    fn parses_latest_npm_version_from_string_or_array() {
        assert_eq!(
            parse_npm_versions(r#""11.18.0""#),
            Some("11.18.0".to_owned())
        );
        assert_eq!(
            parse_npm_versions(r#"["11.17.0","11.18.0","11.9.1"]"#),
            Some("11.18.0".to_owned())
        );
        assert_eq!(parse_npm_versions("{}"), None);
    }
}
