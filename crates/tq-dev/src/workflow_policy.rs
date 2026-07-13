use std::path::{Path, PathBuf};

use crate::error::DevError;
use crate::parse;

const WORKFLOWS_ROOT: &str = ".github/workflows";
const ACTIONS_ROOT: &str = ".github/actions";

#[derive(Debug, Eq, PartialEq)]
enum WorkflowViolationKind {
    MissingTopLevelPermissions,
    MissingJobs,
    UnsupportedTopLevelPermissions(String),
    GlobalWritePermission(String),
    MissingJobTimeout(String),
    InvalidJobTimeout { job: String, value: String },
    JobWriteAll(String),
    NpmLifecycleScripts,
}

#[derive(Debug, Eq, PartialEq)]
struct WorkflowViolation {
    source: PathBuf,
    line: usize,
    kind: WorkflowViolationKind,
}

impl WorkflowViolation {
    fn describe(&self) -> String {
        let message = match &self.kind {
            WorkflowViolationKind::MissingTopLevelPermissions => {
                "workflow must declare top-level permissions".to_owned()
            }
            WorkflowViolationKind::MissingJobs => {
                "workflow must declare at least one job using the supported mapping shape"
                    .to_owned()
            }
            WorkflowViolationKind::UnsupportedTopLevelPermissions(value) => format!(
                "top-level permissions must be a mapping, read-all, or an empty mapping; found {value:?}"
            ),
            WorkflowViolationKind::GlobalWritePermission(permission) => format!(
                "workflow-global {permission} write permission is forbidden; grant it to the owning job"
            ),
            WorkflowViolationKind::MissingJobTimeout(job) => {
                format!("job {job:?} must declare timeout-minutes")
            }
            WorkflowViolationKind::InvalidJobTimeout { job, value } => {
                format!("job {job:?} timeout-minutes must be a positive integer; found {value:?}")
            }
            WorkflowViolationKind::JobWriteAll(job) => {
                format!("job {job:?} must not use permissions: write-all")
            }
            WorkflowViolationKind::NpmLifecycleScripts => {
                "npm ci must include --ignore-scripts in workflow automation".to_owned()
            }
        };
        format!("{}:{}: {message}", self.source.display(), self.line)
    }
}

struct JobState {
    name: String,
    line: usize,
    has_timeout: bool,
    has_valid_timeout: bool,
}

pub fn verify_workflow_hardening(repo_root: &Path) -> Result<(), DevError> {
    let workflow_root = repo_root.join(WORKFLOWS_ROOT);
    let workflow_files = yaml_files(&workflow_root, false)?;
    let mut violations = Vec::new();
    if workflow_files.is_empty() {
        violations.push(WorkflowViolation {
            source: workflow_root,
            line: 1,
            kind: WorkflowViolationKind::MissingTopLevelPermissions,
        });
    }

    for path in &workflow_files {
        let contents = parse::read_to_string(path)?;
        inspect_workflow(&contents, path, &mut violations);
        inspect_npm_installs(&contents, path, &mut violations);
    }

    let actions_root = repo_root.join(ACTIONS_ROOT);
    for path in yaml_files(&actions_root, true)? {
        let contents = parse::read_to_string(&path)?;
        inspect_npm_installs(&contents, &path, &mut violations);
    }

    if violations.is_empty() {
        return Ok(());
    }
    Err(DevError::PolicyViolation {
        details: violations
            .iter()
            .map(WorkflowViolation::describe)
            .collect::<Vec<_>>()
            .join("\n"),
    })
}

fn inspect_workflow(contents: &str, source: &Path, violations: &mut Vec<WorkflowViolation>) {
    let mut has_top_level_permissions = false;
    let mut in_global_permissions = false;
    let mut in_jobs = false;
    let mut has_job = false;
    let mut current_job: Option<JobState> = None;

    for (index, line) in contents.lines().enumerate() {
        let line_number = index + 1;
        let Some((indent, key, value)) = parse::yaml_key_value(line) else {
            continue;
        };

        if indent == 0 {
            finish_job(source, &mut current_job, violations);
            in_global_permissions = false;
            in_jobs = key == "jobs";

            if key == "permissions" {
                has_top_level_permissions = true;
                let value = parse::yaml_scalar(value);
                match value {
                    "" => in_global_permissions = true,
                    "{}" | "read-all" => {}
                    other => violations.push(WorkflowViolation {
                        source: source.to_path_buf(),
                        line: line_number,
                        kind: WorkflowViolationKind::UnsupportedTopLevelPermissions(
                            other.to_owned(),
                        ),
                    }),
                }
            }
            continue;
        }

        if in_global_permissions && indent == 2 {
            if parse::yaml_scalar(value) == "write" {
                violations.push(WorkflowViolation {
                    source: source.to_path_buf(),
                    line: line_number,
                    kind: WorkflowViolationKind::GlobalWritePermission(key.to_owned()),
                });
            }
            continue;
        }

        if !in_jobs {
            continue;
        }
        if indent == 2 {
            finish_job(source, &mut current_job, violations);
            has_job = true;
            current_job = Some(JobState {
                name: key.to_owned(),
                line: line_number,
                has_timeout: false,
                has_valid_timeout: false,
            });
            continue;
        }
        if indent != 4 {
            continue;
        }

        let Some(job) = current_job.as_mut() else {
            continue;
        };
        if key == "timeout-minutes" {
            let timeout = parse::yaml_scalar(value);
            job.has_timeout = true;
            job.has_valid_timeout = timeout.parse::<u64>().is_ok_and(|minutes| minutes > 0);
            if !job.has_valid_timeout {
                violations.push(WorkflowViolation {
                    source: source.to_path_buf(),
                    line: line_number,
                    kind: WorkflowViolationKind::InvalidJobTimeout {
                        job: job.name.clone(),
                        value: timeout.to_owned(),
                    },
                });
            }
        } else if key == "permissions" && parse::yaml_scalar(value) == "write-all" {
            violations.push(WorkflowViolation {
                source: source.to_path_buf(),
                line: line_number,
                kind: WorkflowViolationKind::JobWriteAll(job.name.clone()),
            });
        }
    }

    finish_job(source, &mut current_job, violations);
    if !has_top_level_permissions {
        violations.push(WorkflowViolation {
            source: source.to_path_buf(),
            line: 1,
            kind: WorkflowViolationKind::MissingTopLevelPermissions,
        });
    }
    if !has_job {
        violations.push(WorkflowViolation {
            source: source.to_path_buf(),
            line: 1,
            kind: WorkflowViolationKind::MissingJobs,
        });
    }
}

fn finish_job(
    source: &Path,
    current_job: &mut Option<JobState>,
    violations: &mut Vec<WorkflowViolation>,
) {
    let Some(job) = current_job.take() else {
        return;
    };
    if !job.has_timeout {
        violations.push(WorkflowViolation {
            source: source.to_path_buf(),
            line: job.line,
            kind: WorkflowViolationKind::MissingJobTimeout(job.name),
        });
    }
}

fn inspect_npm_installs(contents: &str, source: &Path, violations: &mut Vec<WorkflowViolation>) {
    for (index, line) in contents.lines().enumerate() {
        if line.contains("npm ci") && !line.contains("--ignore-scripts") {
            violations.push(WorkflowViolation {
                source: source.to_path_buf(),
                line: index + 1,
                kind: WorkflowViolationKind::NpmLifecycleScripts,
            });
        }
    }
}

fn yaml_files(root: &Path, recursive: bool) -> Result<Vec<PathBuf>, DevError> {
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut files = Vec::new();
    collect_yaml_files(root, recursive, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_yaml_files(
    root: &Path,
    recursive: bool,
    files: &mut Vec<PathBuf>,
) -> Result<(), DevError> {
    for entry in std::fs::read_dir(root).map_err(|source| DevError::Io {
        path: root.to_path_buf(),
        source,
    })? {
        let entry = entry.map_err(|source| DevError::Io {
            path: root.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        if recursive && path.is_dir() {
            collect_yaml_files(&path, true, files)?;
        } else if path.extension().is_some_and(|extension| {
            extension.eq_ignore_ascii_case("yml") || extension.eq_ignore_ascii_case("yaml")
        }) {
            files.push(path);
        }
    }
    Ok(())
}
