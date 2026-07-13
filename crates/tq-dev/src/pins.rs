//! Dependency and maintenance-tool drift audits.

use std::path::Path;

use crate::audit::{AuditCheck, AuditCommand, AuditReport, FindingsSignal, run_audit_commands};
use crate::error::DevError;
use crate::invocation::Invocation;
use crate::manifest::{DevToolsManifest, ToolVersion};
use crate::update;

/// Reports available updates for Rust, Cargo, npm, and Python dependencies.
pub fn audit_latest(repo_root: &Path) -> Result<AuditReport, DevError> {
    let manifest = DevToolsManifest::load(repo_root)?;
    let mut checks = vec![
        rust_toolchain_check(repo_root, &manifest.rust),
        mise_pin_check(repo_root, "actionlint", &manifest.actionlint),
        mise_pin_check(repo_root, "shellcheck", &manifest.shellcheck),
    ];
    checks.extend(run_audit_commands(
        repo_root,
        vec![
            AuditCommand {
                name: "cargo-outdated",
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
            },
            AuditCommand {
                name: "npm-outdated",
                invocation: Invocation::new("npm", ["outdated"]),
                signal: FindingsSignal::ExitCodeOne,
            },
            AuditCommand {
                name: "uv-outdated",
                invocation: Invocation::new(
                    "uv",
                    ["pip", "list", "--outdated", "--format", "json"],
                ),
                signal: FindingsSignal::JsonArrayNonEmpty,
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
            },
            AuditCommand {
                name: "cargo-deny",
                invocation: Invocation::new("cargo", ["deny", "check"]),
                signal: FindingsSignal::ExitCodeOne,
            },
            AuditCommand {
                name: "uv-audit",
                invocation: Invocation::new("uv", ["audit", "--locked"]),
                signal: FindingsSignal::ExitCodeOne,
            },
            AuditCommand {
                name: "npm-audit",
                invocation: Invocation::new("npm", ["audit", "--audit-level", "moderate"]),
                signal: FindingsSignal::ExitCodeOne,
            },
        ],
    )?;
    Ok(AuditReport::new(checks))
}

/// Compares pinned Rust maintenance tools against the latest crates.io
/// releases.
pub fn audit_maintenance_tools(repo_root: &Path) -> Result<AuditReport, DevError> {
    let manifest = DevToolsManifest::load(repo_root)?;
    Ok(AuditReport::new(vec![
        crates_io_pin_check(repo_root, "cargo-outdated", &manifest.cargo_outdated),
        crates_io_pin_check(repo_root, "cargo-audit", &manifest.cargo_audit),
        crates_io_pin_check(repo_root, "cargo-deny", &manifest.cargo_deny),
    ]))
}

fn rust_toolchain_check(repo_root: &Path, pinned: &ToolVersion) -> AuditCheck {
    match update::latest_stable_rust(repo_root) {
        Ok(latest) => {
            AuditCheck::pin_comparison("rust", "rustup check", pinned.as_str(), latest.as_str())
        }
        Err(error) => AuditCheck::failed("rust", "rustup check", error.to_string()),
    }
}

fn mise_pin_check(repo_root: &Path, tool: &str, pinned: &ToolVersion) -> AuditCheck {
    let invocation = Invocation::new("mise", ["latest", tool]);
    let command = invocation.display();
    let captured = match invocation.capture(repo_root) {
        Ok(captured) => captured,
        Err(error) => return AuditCheck::failed(tool, &command, error.to_string()),
    };
    if !captured.success {
        return AuditCheck::failed(tool, &command, captured.output);
    }

    let latest = captured.output.trim();
    match ToolVersion::parse(latest) {
        Ok(latest) => AuditCheck::pin_comparison(tool, &command, pinned.as_str(), latest.as_str()),
        Err(message) => AuditCheck::failed(
            tool,
            &command,
            format!("could not parse latest {tool} version: {message}"),
        ),
    }
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
    use super::parse_cargo_search_version;

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
}
