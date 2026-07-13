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
    /// The command returns a JSON array; an empty array is clean, a nonempty
    /// array is findings, and any other output is a failed check.
    JsonArrayNonEmpty,
}

impl FindingsSignal {
    fn classify(self, code: Option<i32>, success: bool, output: &str) -> AuditStatus {
        match self {
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
            Self::JsonArrayNonEmpty => {
                if !success {
                    return AuditStatus::Failed;
                }
                match serde_json::from_str::<Vec<serde_json::Value>>(output) {
                    Ok(values) if values.is_empty() => AuditStatus::Clean,
                    Ok(_) => AuditStatus::Findings,
                    Err(_) => AuditStatus::Failed,
                }
            }
        }
    }
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
        let captured = command.invocation.capture(repo_root)?;
        let status = command
            .signal
            .classify(captured.code, captured.success, &captured.output);

        checks.push(AuditCheck {
            name: command.name.to_owned(),
            command: command.invocation.display(),
            status,
            pinned: None,
            latest: None,
            output: captured.output,
            remediation: (status == AuditStatus::Findings)
                .then(|| command.remediation.map(ToOwned::to_owned))
                .flatten(),
        });
    }

    Ok(checks)
}

#[cfg(test)]
mod tests {
    use super::{AuditStatus, FindingsSignal};

    #[test]
    fn exit_code_one_signal_maps_codes_to_statuses() {
        let signal = FindingsSignal::ExitCodeOne;
        assert_eq!(signal.classify(Some(0), true, ""), AuditStatus::Clean);
        assert_eq!(signal.classify(Some(1), false, ""), AuditStatus::Findings);
        assert_eq!(signal.classify(Some(2), false, ""), AuditStatus::Failed);
        assert_eq!(signal.classify(None, false, ""), AuditStatus::Failed);
    }

    #[test]
    fn output_marker_signal_detects_findings_in_successful_output() {
        let signal = FindingsSignal::OutputContains("latest:");
        assert_eq!(signal.classify(Some(0), true, "ok"), AuditStatus::Clean);
        assert_eq!(
            signal.classify(Some(0), true, "pkg latest: 2.0"),
            AuditStatus::Findings
        );
        assert_eq!(signal.classify(Some(2), false, ""), AuditStatus::Failed);
    }

    #[test]
    fn json_array_signal_fails_closed_on_unrecognized_output() {
        let signal = FindingsSignal::JsonArrayNonEmpty;
        assert_eq!(signal.classify(Some(0), true, "[]"), AuditStatus::Clean);
        assert_eq!(
            signal.classify(Some(0), true, "[{\"name\":\"example\"}]"),
            AuditStatus::Findings
        );
        assert_eq!(
            signal.classify(Some(0), true, "format changed"),
            AuditStatus::Failed
        );
        assert_eq!(signal.classify(Some(2), false, "[]"), AuditStatus::Failed);
    }
}
