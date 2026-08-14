//! Local developer environment diagnosis against the pinned tool manifest.

use std::path::Path;

use serde::Serialize;

use crate::error::DevError;
use crate::invocation;
use crate::label::labeled_enum;
use crate::manifest::{DevToolsManifest, NodeToolchain, ToolVersion};
use crate::native_env;

labeled_enum! {
    pub(crate) enum DoctorStatus {
        Healthy => "healthy",
        Unhealthy => "unhealthy",
    }
}

/// One tool the manifest requires, and what the local machine reports.
#[derive(Debug, Eq, PartialEq, Serialize)]
pub(crate) struct DoctorCheck {
    pub(crate) tool: String,
    pub(crate) assessment: ToolAssessment,
    pub(crate) remediation: Option<String>,
}

impl DoctorCheck {
    pub(crate) fn new(tool: &str, assessment: ToolAssessment, remediation: &str) -> Self {
        Self {
            tool: tool.to_owned(),
            remediation: (!assessment.is_satisfied()).then(|| remediation.to_owned()),
            assessment,
        }
    }
}

/// What a tool check proved about one requirement.
///
/// Reports can be pasted into public issues, so assessments retain only pinned
/// manifest values and parsed versions. Captured output and paths stop at the
/// probe boundary.
#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "requirement", rename_all = "kebab-case")]
pub(crate) enum ToolAssessment {
    Version {
        required: ToolVersion,
        state: VersionState,
    },
    Presence {
        state: PresenceState,
    },
}

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "status", rename_all = "kebab-case")]
pub(crate) enum VersionState {
    #[serde(rename = "ok")]
    Satisfied,
    Mismatched {
        installed: ToolVersion,
    },
    Unreadable,
    Missing,
    Failed,
}

impl VersionState {
    pub(crate) const fn status(&self) -> &'static str {
        match self {
            Self::Satisfied => "ok",
            Self::Mismatched { .. } => "mismatched",
            Self::Unreadable => "unreadable",
            Self::Missing => "missing",
            Self::Failed => "failed",
        }
    }

    const fn is_satisfied(&self) -> bool {
        matches!(self, Self::Satisfied)
    }
}

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "status", rename_all = "kebab-case")]
pub(crate) enum PresenceState {
    #[serde(rename = "ok")]
    Satisfied,
    Missing,
}

impl PresenceState {
    pub(crate) const fn status(&self) -> &'static str {
        match self {
            Self::Satisfied => "ok",
            Self::Missing => "missing",
        }
    }

    const fn is_satisfied(&self) -> bool {
        matches!(self, Self::Satisfied)
    }

    const fn from_presence(is_present: bool) -> Self {
        if is_present {
            Self::Satisfied
        } else {
            Self::Missing
        }
    }
}

impl ToolAssessment {
    pub(crate) const fn is_satisfied(&self) -> bool {
        match self {
            Self::Version { state, .. } => state.is_satisfied(),
            Self::Presence { state } => state.is_satisfied(),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct DoctorReport {
    pub(crate) summary: DoctorSummary,
    pub(crate) checks: Vec<DoctorCheck>,
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
pub(crate) struct DoctorSummary {
    pub(crate) status: DoctorStatus,
    pub(crate) total: usize,
    pub(crate) ok: usize,
    pub(crate) unhealthy: usize,
}

impl DoctorSummary {
    fn from_checks(checks: &[DoctorCheck]) -> Self {
        let ok = checks
            .iter()
            .filter(|check| check.assessment.is_satisfied())
            .count();
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
        ToolAssessment::Version {
            required: expected.clone(),
            state: reported_version_state(expected, program, args),
        },
        remediation,
    )
}

/// Probes `program` and compares the version it reports against the pin. The
/// captured output stops here: only the parsed version escapes.
fn reported_version_state(expected: &ToolVersion, program: &str, args: &[&str]) -> VersionState {
    classify_version_probe(expected, invocation::probe(program, args))
}

fn classify_version_probe(
    expected: &ToolVersion,
    probe: std::io::Result<invocation::Captured>,
) -> VersionState {
    let captured = match probe {
        Ok(captured) => captured,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return VersionState::Missing,
        Err(_) => return VersionState::Failed,
    };
    if !captured.success {
        return VersionState::Failed;
    }
    match version_from_output(&captured.output) {
        None => VersionState::Unreadable,
        Some(installed) if installed == *expected => VersionState::Satisfied,
        Some(installed) => VersionState::Mismatched { installed },
    }
}

fn version_from_output(output: &str) -> Option<ToolVersion> {
    output
        .split(|character: char| !(character.is_ascii_digit() || character == '.'))
        .find_map(|token| ToolVersion::parse(token).ok())
}

/// `uv python find` resolves an interpreter for the pinned version, so a
/// successful probe already proves the pin is satisfied.
fn python_check(expected: &ToolVersion) -> DoctorCheck {
    DoctorCheck::new(
        "python",
        ToolAssessment::Version {
            required: expected.clone(),
            state: classify_python_probe(invocation::probe(
                "uv",
                &["python", "find", expected.as_str()],
            )),
        },
        &format!("Install Python {expected} with uv or the Python manager that owns it."),
    )
}

fn classify_python_probe(probe: std::io::Result<invocation::Captured>) -> VersionState {
    match probe {
        Ok(captured) if captured.success => VersionState::Satisfied,
        Ok(_) => VersionState::Missing,
        Err(_) => VersionState::Failed,
    }
}

fn pkg_config_check() -> DoctorCheck {
    DoctorCheck::new(
        "pkg-config",
        ToolAssessment::Presence {
            state: PresenceState::from_presence(native_env::pkg_config_path().is_some()),
        },
        "Install pkg-config or pkgconf with the system package manager.",
    )
}

fn homebrew_openssl_check() -> DoctorCheck {
    DoctorCheck::new(
        "openssl@3",
        ToolAssessment::Presence {
            state: PresenceState::from_presence(native_env::homebrew_openssl_prefix().is_some()),
        },
        "Install openssl@3 with Homebrew.",
    )
}

#[cfg(test)]
mod tests {
    use std::io;

    use serde_json::json;

    use super::{
        PresenceState, ToolAssessment, VersionState, classify_python_probe, classify_version_probe,
        version_from_output,
    };
    use crate::invocation::Captured;
    use crate::manifest::ToolVersion;

    fn version(text: &str) -> ToolVersion {
        ToolVersion::parse(text).expect("valid version")
    }

    #[test]
    fn version_states_serialize_with_their_rendered_status() {
        for state in [
            VersionState::Satisfied,
            VersionState::Mismatched {
                installed: version("1.95.0"),
            },
            VersionState::Unreadable,
            VersionState::Missing,
            VersionState::Failed,
        ] {
            let serialized = serde_json::to_value(&state).expect("state serializes");
            assert_eq!(serialized["status"], state.status());
        }
    }

    #[test]
    fn assessments_pair_each_requirement_with_only_its_valid_states() {
        assert_eq!(
            serde_json::to_value(ToolAssessment::Version {
                required: version("1.96.1"),
                state: VersionState::Mismatched {
                    installed: version("1.95.0"),
                },
            })
            .expect("assessment serializes"),
            json!({
                "requirement": "version",
                "required": "1.96.1",
                "state": {"status": "mismatched", "installed": "1.95.0"}
            })
        );
        assert_eq!(
            serde_json::to_value(ToolAssessment::Presence {
                state: PresenceState::Satisfied,
            })
            .expect("assessment serializes"),
            json!({"requirement": "presence", "state": {"status": "ok"}})
        );
    }

    #[test]
    fn extracts_only_a_parsed_version_from_command_output() {
        let extract = |output| version_from_output(output).map(|version| version.to_string());

        assert_eq!(
            extract("rustc 1.96.1 (31fca3adb 2026-06-26)"),
            Some("1.96.1".to_owned())
        );
        assert_eq!(extract("v26.4.0"), Some("26.4.0".to_owned()));
        assert_eq!(extract("command not found\ndetails"), None);
        assert_eq!(extract("/Users/example/.local/bin/tool"), None);
    }

    fn captured(success: bool, output: &str) -> Captured {
        Captured {
            code: Some(i32::from(!success)),
            success,
            output: output.to_owned(),
        }
    }

    #[test]
    fn version_probe_outcomes_are_classified_without_leaking_output() {
        let expected = version("1.96.1");

        assert_eq!(
            classify_version_probe(&expected, Ok(captured(true, "rustc 1.96.1"))),
            VersionState::Satisfied
        );
        assert_eq!(
            classify_version_probe(&expected, Ok(captured(true, "rustc 1.95.0"))),
            VersionState::Mismatched {
                installed: version("1.95.0")
            }
        );
        assert_eq!(
            classify_version_probe(&expected, Ok(captured(true, "/private/tool"))),
            VersionState::Unreadable
        );
        assert_eq!(
            classify_version_probe(&expected, Ok(captured(false, "permission denied"))),
            VersionState::Failed
        );
        assert_eq!(
            classify_version_probe(
                &expected,
                Err(io::Error::new(io::ErrorKind::NotFound, "not found")),
            ),
            VersionState::Missing
        );
        assert_eq!(
            classify_version_probe(
                &expected,
                Err(io::Error::new(io::ErrorKind::PermissionDenied, "denied")),
            ),
            VersionState::Failed
        );
    }

    #[test]
    fn python_probe_distinguishes_an_absent_interpreter_from_a_failed_resolver() {
        assert_eq!(
            classify_python_probe(Ok(captured(true, "/private/python"))),
            VersionState::Satisfied
        );
        assert_eq!(
            classify_python_probe(Ok(captured(false, "no interpreter found"))),
            VersionState::Missing
        );
        assert_eq!(
            classify_python_probe(Err(io::Error::new(io::ErrorKind::NotFound, "uv missing"))),
            VersionState::Failed
        );
    }
}
