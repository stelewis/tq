//! Deterministic validation gates: the harness-owned task catalog and
//! runner.

use std::path::Path;
use std::time::Instant;

use serde::Serialize;

use crate::error::DevError;
use crate::invocation::Invocation;
use crate::label::labeled_enum;

labeled_enum! {
    /// Validation surface to check.
    #[derive(clap::ValueEnum)]
    pub enum CheckTarget {
        Routine => "routine",
        Docs => "docs",
        ReleasePolicy => "release-policy",
        ReleaseBuild => "release-build",
        All => "all",
    }
}

labeled_enum! {
    /// Check depth: fast for daily work, full for release-sensitive gates.
    #[derive(clap::ValueEnum)]
    pub enum CheckProfile {
        Fast => "fast",
        Full => "full",
    }
}

labeled_enum! {
    /// The closed catalog of check tasks the harness can run.
    pub enum CheckTask {
        RustFormat => "rust-format",
        RustLint => "rust-lint",
        RustTests => "rust-tests",
        DocsSync => "docs-sync",
        ReleasePolicy => "release-policy",
        ReleaseBuild => "release-build",
    }
}

impl CheckTask {
    #[must_use]
    pub const fn title(self) -> &'static str {
        match self {
            Self::RustFormat => "Rust format",
            Self::RustLint => "Rust lint",
            Self::RustTests => "Rust tests",
            Self::DocsSync => "Generated docs",
            Self::ReleasePolicy => "Release policy",
            Self::ReleaseBuild => "Release build",
        }
    }

    #[must_use]
    pub fn invocation(self) -> Invocation {
        match self {
            Self::RustFormat => Invocation::new("cargo", ["fmt", "--all", "--check"]),
            Self::RustLint => Invocation::new(
                "cargo",
                [
                    "clippy",
                    "--workspace",
                    "--all-targets",
                    "--locked",
                    "--",
                    "-D",
                    "warnings",
                ],
            ),
            Self::RustTests => Invocation::new("cargo", ["test", "--workspace", "--locked"]),
            Self::DocsSync => Invocation::new(
                "cargo",
                [
                    "run",
                    "-p",
                    "tq-docsgen",
                    "--locked",
                    "--",
                    "generate",
                    "all",
                ],
            ),
            Self::ReleasePolicy => Invocation::new(
                "cargo",
                [
                    "run",
                    "-p",
                    "tq-dev",
                    "--locked",
                    "--",
                    "policy",
                    "verify-release",
                    "--repo-root",
                    ".",
                ],
            ),
            Self::ReleaseBuild => Invocation::new(
                "cargo",
                [
                    "run",
                    "-p",
                    "tq-dev",
                    "--locked",
                    "--",
                    "release",
                    "build",
                    "--repo-root",
                    ".",
                ],
            ),
        }
    }
}

/// The resolved plan for a check run.
#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct CheckPlan {
    pub target: CheckTarget,
    pub profile: CheckProfile,
    pub tasks: Vec<CheckTask>,
}

#[must_use]
pub fn plan_checks(target: CheckTarget, profile: CheckProfile) -> CheckPlan {
    let tasks = match target {
        CheckTarget::Routine => routine_tasks(),
        CheckTarget::Docs => vec![CheckTask::DocsSync],
        CheckTarget::ReleasePolicy => vec![CheckTask::ReleasePolicy],
        CheckTarget::ReleaseBuild => vec![CheckTask::ReleaseBuild],
        CheckTarget::All => {
            let mut tasks = routine_tasks();
            tasks.push(CheckTask::DocsSync);
            tasks.push(CheckTask::ReleasePolicy);
            if profile == CheckProfile::Full {
                tasks.push(CheckTask::ReleaseBuild);
            }
            tasks
        }
    };
    CheckPlan {
        target,
        profile,
        tasks,
    }
}

fn routine_tasks() -> Vec<CheckTask> {
    vec![
        CheckTask::RustFormat,
        CheckTask::RustLint,
        CheckTask::RustTests,
    ]
}

labeled_enum! {
    pub enum CheckStatus {
        Passed => "passed",
        Failed => "failed",
    }
}

#[derive(Debug, PartialEq, Serialize)]
pub struct CheckResult {
    pub task: CheckTask,
    pub command: String,
    pub status: CheckStatus,
    pub output: String,
    pub elapsed_seconds: f64,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct CheckReport {
    pub summary: CheckSummary,
    pub checks: Vec<CheckResult>,
}

impl CheckReport {
    #[must_use]
    pub fn passed(&self) -> bool {
        self.summary.status == CheckStatus::Passed
    }
}

#[derive(Debug, PartialEq, Serialize)]
pub struct CheckSummary {
    pub status: CheckStatus,
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub elapsed_seconds: f64,
}

pub fn run_checks(
    repo_root: &Path,
    target: CheckTarget,
    profile: CheckProfile,
) -> Result<CheckReport, DevError> {
    let plan = plan_checks(target, profile);
    let started = Instant::now();
    let mut checks = Vec::new();

    for task in plan.tasks {
        let invocation = task.invocation();
        let task_started = Instant::now();
        let captured = invocation.capture(repo_root)?;

        checks.push(CheckResult {
            task,
            command: invocation.display(),
            status: if captured.success {
                CheckStatus::Passed
            } else {
                CheckStatus::Failed
            },
            output: captured.output,
            elapsed_seconds: task_started.elapsed().as_secs_f64(),
        });
    }

    let passed = checks
        .iter()
        .filter(|check| check.status == CheckStatus::Passed)
        .count();
    let failed = checks.len() - passed;
    Ok(CheckReport {
        summary: CheckSummary {
            status: if failed == 0 {
                CheckStatus::Passed
            } else {
                CheckStatus::Failed
            },
            total: checks.len(),
            passed,
            failed,
            elapsed_seconds: started.elapsed().as_secs_f64(),
        },
        checks,
    })
}
