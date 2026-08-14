//! Local developer environment diagnosis against the pinned tool manifest.

use std::path::Path;

use serde::Serialize;

use crate::error::DevError;
use crate::invocation;
use crate::label::labeled_enum;
use crate::manifest::{DevToolsManifest, NodeToolchain, ToolVersion};
use crate::native_env;

labeled_enum! {
    pub enum DoctorStatus {
        Healthy => "healthy",
        Unhealthy => "unhealthy",
    }
}

/// One tool the manifest requires, and what the local machine reports.
#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct DoctorCheck {
    pub tool: String,
    pub required: ToolRequirement,
    pub state: ToolState,
    pub remediation: Option<String>,
}

impl DoctorCheck {
    /// Remediation is attached only to unmet requirements, so a healthy check
    /// can never carry advice and an unhealthy one can never omit it.
    pub(crate) fn new(
        tool: &str,
        required: ToolRequirement,
        state: ToolState,
        remediation: &str,
    ) -> Self {
        Self {
            tool: tool.to_owned(),
            required,
            remediation: (!state.is_ok()).then(|| remediation.to_owned()),
            state,
        }
    }
}

/// What the manifest requires of a tool.
#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum ToolRequirement {
    /// The tool must report this pinned version.
    Version { version: ToolVersion },
    /// The tool must be installed; the repository pins no version for it.
    Presence,
}

/// What a check observed about the local installation.
///
/// Doctor reports are shared in issue reports and CI logs, so a state carries
/// only manifest vocabulary: a parsed version, or nothing at all. Raw command
/// output and filesystem paths never reach this type.
#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "status", rename_all = "kebab-case")]
pub enum ToolState {
    /// The requirement is met.
    Ok,
    /// The tool is installed but reports a different version.
    Mismatched { installed: ToolVersion },
    /// The tool ran but reported no version, so the pin cannot be verified.
    Unreadable,
    /// The tool is not installed, or could not run.
    Missing,
}

impl ToolState {
    /// The status label shared by rendered tables and the serialized `status`
    /// tag.
    #[must_use]
    pub(crate) const fn label(&self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Mismatched { .. } => "mismatched",
            Self::Unreadable => "unreadable",
            Self::Missing => "missing",
        }
    }

    #[must_use]
    pub(crate) const fn is_ok(&self) -> bool {
        matches!(self, Self::Ok)
    }

    const fn present(is_present: bool) -> Self {
        if is_present { Self::Ok } else { Self::Missing }
    }
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct DoctorReport {
    pub summary: DoctorSummary,
    pub checks: Vec<DoctorCheck>,
}

impl DoctorReport {
    #[must_use]
    pub fn is_healthy(&self) -> bool {
        self.summary.status == DoctorStatus::Healthy
    }

    /// The only way to build a report: the summary is always derived from the
    /// checks it describes.
    pub(crate) fn new(checks: Vec<DoctorCheck>) -> Self {
        Self {
            summary: DoctorSummary::from_checks(&checks),
            checks,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct DoctorSummary {
    pub status: DoctorStatus,
    pub total: usize,
    pub ok: usize,
    pub unhealthy: usize,
}

impl DoctorSummary {
    fn from_checks(checks: &[DoctorCheck]) -> Self {
        let ok = checks.iter().filter(|check| check.state.is_ok()).count();
        let unhealthy = checks.len() - ok;

        Self {
            status: if unhealthy == 0 {
                DoctorStatus::Healthy
            } else {
                DoctorStatus::Unhealthy
            },
            total: checks.len(),
            ok,
            unhealthy,
        }
    }
}

pub fn diagnose_automation(repo_root: &Path) -> Result<DoctorReport, DevError> {
    let manifest = DevToolsManifest::load(repo_root)?;
    Ok(DoctorReport::new(vec![
        version_check(
            "actionlint",
            &manifest.actionlint,
            "actionlint",
            &["-version"],
            "Update actionlint with the environment manager that owns it.",
        ),
        version_check(
            "shellcheck",
            &manifest.shellcheck,
            "shellcheck",
            &["--version"],
            "Update ShellCheck with the environment manager that owns it.",
        ),
    ]))
}

pub fn diagnose(repo_root: &Path) -> Result<DoctorReport, DevError> {
    let manifest = DevToolsManifest::load(repo_root)?;
    let node = NodeToolchain::load(repo_root)?;
    let mut checks = vec![
        version_check(
            "rustc",
            &manifest.rust,
            "rustc",
            &["--version"],
            "Update the selected Rust toolchain with rustup or its owning environment manager.",
        ),
        version_check(
            "cargo",
            &manifest.rust,
            "cargo",
            &["--version"],
            "Update the selected Rust toolchain with rustup or its owning environment manager.",
        ),
        version_check(
            "uv",
            &manifest.uv,
            "uv",
            &["--version"],
            "Update uv with the package manager that installed it.",
        ),
        version_check(
            "node",
            &node.node,
            "node",
            &["--version"],
            "Update Node with the environment manager that owns it.",
        ),
        version_check(
            "npm",
            &node.npm,
            "npm",
            &["--version"],
            "Reinstall the pinned Node release to restore its bundled npm version.",
        ),
        version_check(
            "actionlint",
            &manifest.actionlint,
            "actionlint",
            &["-version"],
            "Update actionlint with the environment manager that owns it.",
        ),
        version_check(
            "shellcheck",
            &manifest.shellcheck,
            "shellcheck",
            &["--version"],
            "Update ShellCheck with the environment manager that owns it.",
        ),
        python_check(&manifest.python),
        version_check(
            "cargo-outdated",
            &manifest.cargo_outdated,
            "cargo",
            &["outdated", "--version"],
            "Install the pinned cargo-outdated version with Cargo or your environment manager.",
        ),
        version_check(
            "cargo-deny",
            &manifest.cargo_deny,
            "cargo",
            &["deny", "--version"],
            "Install the pinned cargo-deny version with Cargo or your environment manager.",
        ),
        version_check(
            "cargo-audit",
            &manifest.cargo_audit,
            "cargo",
            &["audit", "--version"],
            "Install the pinned cargo-audit version with Cargo or your environment manager.",
        ),
        pkg_config_check(),
    ];
    if cfg!(target_os = "macos") {
        checks.push(homebrew_openssl_check());
    }

    Ok(DoctorReport::new(checks))
}

fn version_check(
    tool: &str,
    expected: &ToolVersion,
    program: &str,
    args: &[&str],
    remediation: &str,
) -> DoctorCheck {
    DoctorCheck::new(
        tool,
        ToolRequirement::Version {
            version: expected.clone(),
        },
        reported_version_state(expected, program, args),
        remediation,
    )
}

/// Probes `program` and compares the version it reports against the pin. The
/// captured output stops here: only the parsed version escapes.
fn reported_version_state(expected: &ToolVersion, program: &str, args: &[&str]) -> ToolState {
    let Ok(captured) = invocation::probe(program, args) else {
        return ToolState::Missing;
    };
    if !captured.success {
        return ToolState::Missing;
    }
    match ToolVersion::from_version_output(&captured.output) {
        None => ToolState::Unreadable,
        Some(installed) if installed == *expected => ToolState::Ok,
        Some(installed) => ToolState::Mismatched { installed },
    }
}

/// `uv python find` resolves an interpreter for the pinned version, so a
/// successful probe already proves the pin is satisfied.
fn python_check(expected: &ToolVersion) -> DoctorCheck {
    let found = invocation::probe("uv", &["python", "find", expected.as_str()])
        .is_ok_and(|captured| captured.success);
    DoctorCheck::new(
        "python",
        ToolRequirement::Version {
            version: expected.clone(),
        },
        ToolState::present(found),
        &format!("Install Python {expected} with uv or the Python manager that owns it."),
    )
}

fn pkg_config_check() -> DoctorCheck {
    DoctorCheck::new(
        "pkg-config",
        ToolRequirement::Presence,
        ToolState::present(native_env::pkg_config_path().is_some()),
        "Install pkg-config or pkgconf with the system package manager.",
    )
}

fn homebrew_openssl_check() -> DoctorCheck {
    DoctorCheck::new(
        "openssl@3",
        ToolRequirement::Presence,
        ToolState::present(native_env::homebrew_openssl_prefix().is_some()),
        "Install openssl@3 with Homebrew.",
    )
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{ToolRequirement, ToolState};
    use crate::manifest::ToolVersion;

    fn version(text: &str) -> ToolVersion {
        ToolVersion::parse(text).expect("valid version")
    }

    #[test]
    fn states_serialize_with_their_rendered_status_label() {
        for state in [
            ToolState::Ok,
            ToolState::Mismatched {
                installed: version("1.95.0"),
            },
            ToolState::Unreadable,
            ToolState::Missing,
        ] {
            let serialized = serde_json::to_value(&state).expect("state serializes");
            assert_eq!(serialized["status"], state.label());
        }
    }

    #[test]
    fn states_carry_manifest_vocabulary_only() {
        assert_eq!(
            serde_json::to_value(ToolState::Mismatched {
                installed: version("1.95.0"),
            })
            .expect("state serializes"),
            json!({"status": "mismatched", "installed": "1.95.0"})
        );
        assert_eq!(
            serde_json::to_value(ToolRequirement::Version {
                version: version("1.96.1"),
            })
            .expect("requirement serializes"),
            json!({"kind": "version", "version": "1.96.1"})
        );
        assert_eq!(
            serde_json::to_value(ToolRequirement::Presence).expect("requirement serializes"),
            json!({"kind": "presence"})
        );
    }
}
