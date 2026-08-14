//! Audit reports: run read-only checks and classify each result as clean,
//! findings, or failed.

use std::path::Path;

use serde::Serialize;

use crate::error::DevError;
use crate::invocation::Invocation;
use crate::label::labeled_enum;

labeled_enum! {
    pub enum AuditStatus {
        Clean => "clean",
        Findings => "findings",
        Failed => "failed",
    }
}

/// How a command's captured result maps to an [`AuditStatus`], declared
/// where the command is defined.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FindingsSignal {
    /// Exit code 0 is clean, exit code 1 is findings, anything else failed.
    ExitCodeOne,
    /// The command exits 0 even with findings; findings are detected by a
    /// marker in the output.
    OutputContains(&'static str),
    /// Cargo update dry-run output reports `Locking N package(s)` and exits 0
    /// for both clean and stale locks.
    CargoUpdateDryRun,
    /// Cargo deny exits 1 for both policy findings and execution failures;
    /// findings use its stable `error[category]:` diagnostic form.
    CargoDeny,
    /// Cargo audit exits 1 for both advisory findings and execution failures;
    /// findings end with its stable vulnerability or denied-warning summary.
    CargoAudit,
    /// The command returns a JSON array of outdated packages with `name`,
    /// `version`, and `latest_version` fields; an empty array is clean and
    /// any other output shape is a failed check.
    OutdatedPackagesJson,
}

/// A classified command result: the audit status plus the output to report.
struct Classified {
    status: AuditStatus,
    output: String,
}

impl FindingsSignal {
    fn classify(self, code: Option<i32>, success: bool, output: String) -> Classified {
        let status = match self {
            Self::ExitCodeOne => {
                if success {
                    AuditStatus::Clean
                } else if code == Some(1) {
                    AuditStatus::Findings
                } else {
                    AuditStatus::Failed
                }
            }
            Self::OutputContains(marker) => {
                if !success {
                    AuditStatus::Failed
                } else if output.contains(marker) {
                    AuditStatus::Findings
                } else {
                    AuditStatus::Clean
                }
            }
            Self::CargoUpdateDryRun => {
                if success {
                    match parse_cargo_update_count(&output) {
                        Some(0) => AuditStatus::Clean,
                        Some(_) => AuditStatus::Findings,
                        None => AuditStatus::Failed,
                    }
                } else {
                    AuditStatus::Failed
                }
            }
            Self::CargoDeny => {
                if success {
                    AuditStatus::Clean
                } else if code == Some(1) && output.lines().any(|line| line.starts_with("error[")) {
                    AuditStatus::Findings
                } else {
                    AuditStatus::Failed
                }
            }
            Self::CargoAudit => {
                if success {
                    AuditStatus::Clean
                } else if code == Some(1)
                    && output.lines().any(|line| {
                        line.starts_with("error: ")
                            && (line.ends_with(" vulnerabilities found!")
                                || line.ends_with(" vulnerability found!")
                                || line.ends_with(" denied warnings found!")
                                || line.ends_with(" denied warning found!"))
                    })
                {
                    AuditStatus::Findings
                } else {
                    AuditStatus::Failed
                }
            }
            Self::OutdatedPackagesJson => {
                if success {
                    match parse_outdated_packages(&output) {
                        Some(packages) if packages.is_empty() => {
                            return Classified {
                                status: AuditStatus::Clean,
                                output: String::new(),
                            };
                        }
                        Some(packages) => {
                            return Classified {
                                status: AuditStatus::Findings,
                                output: packages.join("\n"),
                            };
                        }
                        None => AuditStatus::Failed,
                    }
                } else {
                    AuditStatus::Failed
                }
            }
        };
        Classified { status, output }
    }
}

fn parse_cargo_update_count(output: &str) -> Option<usize> {
    output.lines().find_map(|line| {
        line.trim()
            .strip_prefix("Locking ")?
            .split_whitespace()
            .next()?
            .parse()
            .ok()
    })
}

/// Parses an outdated-packages JSON array into `name version -> latest`
/// lines, returning `None` when the output shape is unrecognized.
fn parse_outdated_packages(output: &str) -> Option<Vec<String>> {
    #[derive(serde::Deserialize)]
    struct OutdatedPackage {
        name: String,
        version: String,
        latest_version: String,
    }

    serde_json::from_str::<Vec<OutdatedPackage>>(output)
        .ok()
        .map(|packages| {
            packages
                .into_iter()
                .map(|package| {
                    format!(
                        "{} {} -> {}",
                        package.name, package.version, package.latest_version
                    )
                })
                .collect()
        })
}

/// An audit check to run: a named invocation plus its findings policy.
#[derive(Debug)]
pub struct AuditCommand {
    pub name: &'static str,
    pub invocation: Invocation,
    pub signal: FindingsSignal,
    pub remediation: Option<&'static str>,
}

/// The outcome of a single audit check.
#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct AuditCheck {
    pub name: String,
    pub command: String,
    pub status: AuditStatus,
    /// Pinned version, when the check compares a pin against upstream.
    pub pinned: Option<String>,
    /// Latest upstream version, when the check compares a pin against
    /// upstream.
    pub latest: Option<String>,
    pub output: String,
    pub remediation: Option<String>,
}

impl AuditCheck {
    #[must_use]
    pub fn clean(name: &str, command: &str) -> Self {
        Self {
            name: name.to_owned(),
            command: command.to_owned(),
            status: AuditStatus::Clean,
            pinned: None,
            latest: None,
            output: String::new(),
            remediation: None,
        }
    }

    #[must_use]
    pub fn failed(name: &str, command: &str, output: String) -> Self {
        Self {
            name: name.to_owned(),
            command: command.to_owned(),
            status: AuditStatus::Failed,
            pinned: None,
            latest: None,
            output,
            remediation: None,
        }
    }

    /// A pin-drift comparison: clean when pinned matches latest.
    #[must_use]
    pub fn pin_comparison(name: &str, command: &str, pinned: &str, latest: &str) -> Self {
        let status = if pinned == latest {
            AuditStatus::Clean
        } else {
            AuditStatus::Findings
        };
        Self {
            name: name.to_owned(),
            command: command.to_owned(),
            status,
            pinned: Some(pinned.to_owned()),
            latest: Some(latest.to_owned()),
            output: String::new(),
            remediation: None,
        }
    }

    #[must_use]
    pub fn with_remediation(mut self, remediation: &str) -> Self {
        if self.status != AuditStatus::Clean {
            self.remediation = Some(remediation.to_owned());
        }
        self
    }
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct AuditReport {
    pub summary: AuditSummary,
    pub checks: Vec<AuditCheck>,
}

impl AuditReport {
    #[must_use]
    pub fn new(checks: Vec<AuditCheck>) -> Self {
        Self {
            summary: AuditSummary::from_checks(&checks),
            checks,
        }
    }

    #[must_use]
    pub fn has_findings(&self) -> bool {
        self.summary.status != AuditStatus::Clean
    }
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct AuditSummary {
    pub status: AuditStatus,
    pub total: usize,
    pub clean: usize,
    pub findings: usize,
    pub failed: usize,
}

impl AuditSummary {
    fn from_checks(checks: &[AuditCheck]) -> Self {
        let count =
            |status: AuditStatus| checks.iter().filter(|check| check.status == status).count();
        let clean = count(AuditStatus::Clean);
        let findings = count(AuditStatus::Findings);
        let failed = count(AuditStatus::Failed);

        let status = if failed > 0 {
            AuditStatus::Failed
        } else if findings > 0 {
            AuditStatus::Findings
        } else {
            AuditStatus::Clean
        };

        Self {
            status,
            total: checks.len(),
            clean,
            findings,
            failed,
        }
    }
}

/// Runs audit commands, classifying each captured result with its declared
/// findings signal.
pub fn run_audit_commands(
    repo_root: &Path,
    commands: Vec<AuditCommand>,
) -> Result<Vec<AuditCheck>, DevError> {
    let mut checks = Vec::new();

    for command in commands {
        checks.push(run_audit_command(
            repo_root,
            command.name,
            &command.invocation,
            command.signal,
            command.remediation,
        )?);
    }

    Ok(checks)
}

pub fn run_audit_command(
    repo_root: &Path,
    name: &str,
    invocation: &Invocation,
    signal: FindingsSignal,
    remediation: Option<&str>,
) -> Result<AuditCheck, DevError> {
    let captured = invocation.capture(repo_root)?;
    let classified = signal.classify(captured.code, captured.success, captured.output);

    Ok(AuditCheck {
        name: name.to_owned(),
        command: invocation.display(),
        status: classified.status,
        pinned: None,
        latest: None,
        output: classified.output,
        remediation: (classified.status == AuditStatus::Findings)
            .then(|| remediation.map(ToOwned::to_owned))
            .flatten(),
    })
}

#[cfg(test)]
mod tests {
    use super::{AuditStatus, FindingsSignal};

    fn status(
        signal: FindingsSignal,
        code: Option<i32>,
        success: bool,
        output: &str,
    ) -> AuditStatus {
        signal.classify(code, success, output.to_owned()).status
    }

    #[test]
    fn exit_code_one_signal_maps_codes_to_statuses() {
        let signal = FindingsSignal::ExitCodeOne;
        assert_eq!(status(signal, Some(0), true, ""), AuditStatus::Clean);
        assert_eq!(status(signal, Some(1), false, ""), AuditStatus::Findings);
        assert_eq!(status(signal, Some(2), false, ""), AuditStatus::Failed);
        assert_eq!(status(signal, None, false, ""), AuditStatus::Failed);
    }

    #[test]
    fn output_marker_signal_detects_findings_in_successful_output() {
        let signal = FindingsSignal::OutputContains("latest:");
        assert_eq!(status(signal, Some(0), true, "ok"), AuditStatus::Clean);
        assert_eq!(
            status(signal, Some(0), true, "pkg latest: 2.0"),
            AuditStatus::Findings
        );
        assert_eq!(status(signal, Some(2), false, ""), AuditStatus::Failed);
    }

    #[test]
    fn cargo_update_signal_distinguishes_zero_from_planned_updates() {
        let signal = FindingsSignal::CargoUpdateDryRun;
        assert_eq!(
            status(
                signal,
                Some(0),
                true,
                "Locking 0 packages to latest versions"
            ),
            AuditStatus::Clean
        );
        assert_eq!(
            status(
                signal,
                Some(0),
                true,
                "Locking 3 packages to latest versions"
            ),
            AuditStatus::Findings
        );
        assert_eq!(
            status(signal, Some(0), true, "format changed"),
            AuditStatus::Failed
        );
        assert_eq!(
            status(signal, Some(1), false, "Locking 3 packages"),
            AuditStatus::Failed
        );
    }

    #[test]
    fn cargo_deny_signal_separates_findings_from_execution_errors() {
        let signal = FindingsSignal::CargoDeny;
        assert_eq!(
            status(signal, Some(0), true, "advisories ok"),
            AuditStatus::Clean
        );
        assert_eq!(
            status(
                signal,
                Some(1),
                false,
                "error[vulnerability]: affected crate"
            ),
            AuditStatus::Findings
        );
        assert_eq!(
            status(signal, Some(1), false, "[ERROR] cargo metadata failed"),
            AuditStatus::Failed
        );
        assert_eq!(
            status(signal, Some(2), false, "error[policy]: bad"),
            AuditStatus::Failed
        );
    }

    #[test]
    fn cargo_audit_signal_separates_findings_from_execution_errors() {
        let signal = FindingsSignal::CargoAudit;
        assert_eq!(status(signal, Some(0), true, ""), AuditStatus::Clean);
        assert_eq!(
            status(signal, Some(1), false, "error: 2 vulnerabilities found!"),
            AuditStatus::Findings
        );
        assert_eq!(
            status(signal, Some(1), false, "error: 1 denied warning found!"),
            AuditStatus::Findings
        );
        assert_eq!(
            status(
                signal,
                Some(1),
                false,
                "error: advisory database unavailable"
            ),
            AuditStatus::Failed
        );
    }

    #[test]
    fn outdated_packages_signal_renders_findings_and_fails_closed() {
        let signal = FindingsSignal::OutdatedPackagesJson;
        assert_eq!(status(signal, Some(0), true, "[]"), AuditStatus::Clean);

        let classified = signal.classify(
            Some(0),
            true,
            r#"[{"name":"ruff","version":"0.15.20","latest_version":"0.15.21"}]"#.to_owned(),
        );
        assert_eq!(classified.status, AuditStatus::Findings);
        assert_eq!(classified.output, "ruff 0.15.20 -> 0.15.21");

        assert_eq!(
            status(signal, Some(0), true, "format changed"),
            AuditStatus::Failed
        );
        assert_eq!(status(signal, Some(2), false, "[]"), AuditStatus::Failed);
    }
}
