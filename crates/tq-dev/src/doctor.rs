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

labeled_enum! {
    pub enum ToolStatus {
        Ok => "ok",
        Missing => "missing",
        Mismatched => "mismatched",
    }
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct DoctorCheck {
    pub tool: String,
    pub expected: String,
    pub actual: Option<String>,
    pub status: ToolStatus,
    pub remediation: Option<String>,
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

    fn new(checks: Vec<DoctorCheck>) -> Self {
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
    pub missing: usize,
    pub mismatched: usize,
}

impl DoctorSummary {
    fn from_checks(checks: &[DoctorCheck]) -> Self {
        let count =
            |status: ToolStatus| checks.iter().filter(|check| check.status == status).count();
        let ok = count(ToolStatus::Ok);
        let missing = count(ToolStatus::Missing);
        let mismatched = count(ToolStatus::Mismatched);

        Self {
            status: if missing == 0 && mismatched == 0 {
                DoctorStatus::Healthy
            } else {
                DoctorStatus::Unhealthy
            },
            total: checks.len(),
            ok,
            missing,
            mismatched,
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
            "Update npm with the Node toolchain manager that owns it.",
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
    match invocation::probe(program, args) {
        Ok(captured) if captured.success => {
            let status = if expected.matches_version_output(&captured.output) {
                ToolStatus::Ok
            } else {
                ToolStatus::Mismatched
            };
            DoctorCheck {
                tool: tool.to_owned(),
                expected: expected.as_str().to_owned(),
                actual: Some(captured.output),
                remediation: (status != ToolStatus::Ok).then(|| remediation.to_owned()),
                status,
            }
        }
        Ok(captured) => DoctorCheck {
            tool: tool.to_owned(),
            expected: expected.as_str().to_owned(),
            actual: Some(captured.output),
            status: ToolStatus::Missing,
            remediation: Some(remediation.to_owned()),
        },
        Err(_) => DoctorCheck {
            tool: tool.to_owned(),
            expected: expected.as_str().to_owned(),
            actual: None,
            status: ToolStatus::Missing,
            remediation: Some(remediation.to_owned()),
        },
    }
}

fn python_check(expected: &ToolVersion) -> DoctorCheck {
    let remediation = format!(
        "Install Python {} with uv or the Python manager that owns it.",
        expected.as_str()
    );
    match invocation::probe("uv", &["python", "find", expected.as_str()]) {
        Ok(captured) if captured.success => DoctorCheck {
            tool: "python".to_owned(),
            expected: expected.as_str().to_owned(),
            actual: Some(captured.output),
            status: ToolStatus::Ok,
            remediation: None,
        },
        Ok(captured) => DoctorCheck {
            tool: "python".to_owned(),
            expected: expected.as_str().to_owned(),
            actual: Some(captured.output),
            status: ToolStatus::Missing,
            remediation: Some(remediation),
        },
        Err(_) => DoctorCheck {
            tool: "python".to_owned(),
            expected: expected.as_str().to_owned(),
            actual: None,
            status: ToolStatus::Missing,
            remediation: Some(remediation),
        },
    }
}

fn pkg_config_check() -> DoctorCheck {
    let expected = "pkg-config or pkgconf on PATH".to_owned();
    match native_env::pkg_config_path() {
        Some(path) => DoctorCheck {
            tool: "pkg-config".to_owned(),
            expected,
            actual: Some(path),
            status: ToolStatus::Ok,
            remediation: None,
        },
        None => DoctorCheck {
            tool: "pkg-config".to_owned(),
            expected,
            actual: None,
            status: ToolStatus::Missing,
            remediation: Some(
                "Install pkg-config or pkgconf with the system package manager.".to_owned(),
            ),
        },
    }
}

fn homebrew_openssl_check() -> DoctorCheck {
    let expected = "Homebrew openssl@3".to_owned();
    match native_env::homebrew_openssl_prefix() {
        Some(path) => DoctorCheck {
            tool: "openssl@3".to_owned(),
            expected,
            actual: Some(path),
            status: ToolStatus::Ok,
            remediation: None,
        },
        None => DoctorCheck {
            tool: "openssl@3".to_owned(),
            expected,
            actual: None,
            status: ToolStatus::Missing,
            remediation: Some("Install openssl@3 with Homebrew.".to_owned()),
        },
    }
}
