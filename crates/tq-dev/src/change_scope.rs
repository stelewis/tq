use std::collections::BTreeSet;
use std::fmt;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use crate::error::DevError;

const ZERO_SHA: &str = "0000000000000000000000000000000000000000";

#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct ChangedPath(PathBuf);

impl ChangedPath {
    pub fn parse(value: impl Into<PathBuf>) -> Result<Self, DevError> {
        let value = value.into();
        let is_valid = !value.as_os_str().is_empty()
            && !value.is_absolute()
            && value
                .components()
                .all(|component| matches!(component, Component::Normal(_)))
            && !value.to_string_lossy().contains('\\');
        if !is_valid {
            return Err(DevError::InvalidInput {
                path: value,
                message: "changed path must be a normalized repository-relative path".to_owned(),
            });
        }
        Ok(Self(value))
    }

    #[must_use]
    pub fn as_path(&self) -> &Path {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ChangeSource<'a> {
    All,
    PullRequest {
        base_ref: &'a str,
        head_ref: &'a str,
    },
    Push {
        base_ref: &'a str,
        head_ref: &'a str,
    },
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum GateDecision {
    Skip,
    Run,
}

impl GateDecision {
    #[must_use]
    pub const fn should_run(self) -> bool {
        matches!(self, Self::Run)
    }

    const fn enable_if(&mut self, condition: bool) {
        if condition {
            *self = Self::Run;
        }
    }
}

impl fmt::Display for GateDecision {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.should_run().fmt(formatter)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ChangeScope {
    pub docs_only: bool,
    pub runtime_dependency_inputs: GateDecision,
    pub rust_security: GateDecision,
    pub docs_security: GateDecision,
    pub release_relevant: GateDecision,
    pub docs_sync: GateDecision,
    pub docs_build: GateDecision,
    pub broad_gate: GateDecision,
    pub unknown_paths: BTreeSet<ChangedPath>,
}

impl ChangeScope {
    #[must_use]
    pub const fn all() -> Self {
        Self {
            docs_only: false,
            runtime_dependency_inputs: GateDecision::Run,
            rust_security: GateDecision::Run,
            docs_security: GateDecision::Run,
            release_relevant: GateDecision::Run,
            docs_sync: GateDecision::Run,
            docs_build: GateDecision::Run,
            broad_gate: GateDecision::Run,
            unknown_paths: BTreeSet::new(),
        }
    }

    #[must_use]
    pub fn github_output(&self) -> String {
        format!(
            concat!(
                "docs_only={}\n",
                "runtime_dependency_inputs={}\n",
                "rust_security={}\n",
                "docs_security={}\n",
                "release_relevant={}\n",
                "docs_sync={}\n",
                "docs_build={}\n",
                "broad_gate={}\n",
            ),
            self.docs_only,
            self.runtime_dependency_inputs,
            self.rust_security,
            self.docs_security,
            self.release_relevant,
            self.docs_sync,
            self.docs_build,
            self.broad_gate,
        )
    }
}

pub fn classify_git_changes(
    repo_root: &Path,
    source: ChangeSource<'_>,
) -> Result<ChangeScope, DevError> {
    let (base_ref, head_ref) = match source {
        ChangeSource::All => return Ok(ChangeScope::all()),
        ChangeSource::Push {
            base_ref,
            head_ref: _,
        } if base_ref == ZERO_SHA => return Ok(ChangeScope::all()),
        ChangeSource::Push { base_ref, head_ref } => (base_ref.to_owned(), head_ref),
        ChangeSource::PullRequest { base_ref, head_ref } => {
            let Some(merge_base) = try_git_stdout(repo_root, &["merge-base", base_ref, head_ref])?
            else {
                return Ok(ChangeScope::all());
            };
            (merge_base.trim().to_owned(), head_ref)
        }
    };

    classify_paths(git_changed_paths(repo_root, &base_ref, head_ref)?)
}

pub fn verify_tracked_path_coverage(repo_root: &Path) -> Result<(), DevError> {
    let args = ["ls-files", "-z"];
    let output = git_output(repo_root, &args)?;
    if !output.status.success() {
        return Err(DevError::Git {
            args: args.join(" "),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        });
    }

    let scope = classify_paths(parse_nul_paths(&output.stdout)?)?;
    if scope.unknown_paths.is_empty() {
        return Ok(());
    }

    let details = scope
        .unknown_paths
        .iter()
        .map(|path| format!("- {}", path.as_path().display()))
        .collect::<Vec<_>>()
        .join("\n");
    Err(DevError::PolicyViolation {
        details: format!(
            "tracked paths are missing from the change-scope contract:\n{details}\nclassify each path before merging"
        ),
    })
}

pub fn classify_paths(
    paths: impl IntoIterator<Item = ChangedPath>,
) -> Result<ChangeScope, DevError> {
    let paths = paths.into_iter().collect::<Vec<_>>();
    let mut scope = ChangeScope {
        docs_only: !paths.is_empty() && paths.iter().all(|path| is_documentation(path.as_path())),
        runtime_dependency_inputs: GateDecision::Skip,
        rust_security: GateDecision::Skip,
        docs_security: GateDecision::Skip,
        release_relevant: GateDecision::Skip,
        docs_sync: GateDecision::Skip,
        docs_build: GateDecision::Skip,
        broad_gate: GateDecision::Skip,
        unknown_paths: BTreeSet::new(),
    };

    for path in paths {
        classify_path(&path, &mut scope);
    }

    if !scope.unknown_paths.is_empty() {
        let unknown_paths = scope.unknown_paths;
        scope = ChangeScope::all();
        scope.unknown_paths = unknown_paths;
    }

    Ok(scope)
}

fn classify_path(path: &ChangedPath, scope: &mut ChangeScope) {
    let value = path.as_path().to_string_lossy();

    scope
        .runtime_dependency_inputs
        .enable_if(is_cargo_dependency_input(&value));
    scope
        .rust_security
        .enable_if(is_rust_security_input(&value));
    scope
        .docs_security
        .enable_if(is_docs_security_input(&value));
    scope.release_relevant.enable_if(is_release_input(&value));
    scope.docs_sync.enable_if(is_docs_sync_input(&value));
    scope.docs_build.enable_if(is_docs_build_input(&value));

    if !is_known_path(&value) {
        scope.unknown_paths.insert(path.clone());
    }
}

fn is_documentation(path: &Path) -> bool {
    path.starts_with("docs")
        || matches!(
            path.to_string_lossy().as_ref(),
            "README.md" | "CHANGELOG.md" | "CONTRIBUTING.md" | "SECURITY.md"
        )
}

fn is_cargo_dependency_input(path: &str) -> bool {
    matches!(path, "Cargo.lock" | "Cargo.toml")
        || (path.starts_with("crates/") && path.ends_with("/Cargo.toml"))
}

fn is_rust_security_input(path: &str) -> bool {
    is_cargo_dependency_input(path)
        || path == "deny.toml"
        || matches!(
            path,
            ".github/workflows/ci.yml"
                | ".github/workflows/rust-security-advisories.yml"
                | ".github/actions/setup-rust/action.yml"
                | ".github/actions/setup-rust-security-tools/action.yml"
                | ".github/actions/setup-rust-maintenance-tools/action.yml"
        )
}

fn is_docs_security_input(path: &str) -> bool {
    matches!(
        path,
        "package.json"
            | "package-lock.json"
            | ".github/workflows/ci.yml"
            | ".github/workflows/docs-security.yml"
            | ".github/workflows/docs-pages.yml"
            | ".github/actions/setup-node/action.yml"
            | ".github/actions/setup-docs/action.yml"
    )
}

fn is_docs_sync_input(path: &str) -> bool {
    matches!(
        path,
        "Cargo.toml"
            | "pyproject.toml"
            | "README.md"
            | ".github/workflows/ci.yml"
            | "docs/reference/cli/options-manifest.json"
            | "docs/reference/config/examples-manifest.json"
            | "docs/reference/cli.md"
            | "docs/reference/configuration.md"
            | "docs/guide/quickstart.md"
            | "docs/.vitepress/generated/rules-sidebar.ts"
    ) || path.starts_with("crates/tq-docsgen/")
        || path.starts_with("crates/tq-cli/")
        || path.starts_with("crates/tq-rules/")
        || (path.starts_with("docs/reference/rules/")
            && Path::new(path)
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("md")))
}

fn is_docs_build_input(path: &str) -> bool {
    path.starts_with("docs/")
        || path.starts_with("crates/tq-docsgen/")
        || path.starts_with("crates/tq-cli/")
        || path.starts_with("crates/tq-rules/")
        || matches!(
            path,
            "package.json"
                | "package-lock.json"
                | ".github/workflows/ci.yml"
                | ".github/workflows/docs-pages.yml"
                | ".github/actions/setup-node/action.yml"
                | ".github/actions/setup-docs/action.yml"
        )
}

fn is_release_input(path: &str) -> bool {
    is_cargo_dependency_input(path)
        || path.starts_with("crates/tq-release/")
        || matches!(
            path,
            "pyproject.toml"
                | "uv.lock"
                | ".github/dev-tools.toml"
                | ".github/workflows/ci.yml"
                | ".github/workflows/publish.yml"
                | "crates/tq-dev/src/artifacts.rs"
                | "crates/tq-dev/src/release.rs"
        )
}

fn is_known_path(path: &str) -> bool {
    is_known_crate_path(path)
        || path.starts_with("docs/")
        || path.starts_with(".github/instructions/")
        || path.starts_with(".github/prompts/")
        || path.starts_with(".github/skills/")
        || path.starts_with(".github/dependabot/")
        || path.starts_with(".github/ISSUE_TEMPLATE/")
        || path.starts_with(".github/PULL_REQUEST_TEMPLATE/")
        || path.starts_with(".cargo/")
        || matches!(
            path,
            ".copier-answers.yml"
                | ".detect-secrets.baseline"
                | ".gitattributes"
                | ".gitignore"
                | ".gitleaks.toml"
                | ".lycheeignore"
                | ".markdownlint.yaml"
                | ".pre-commit-config.yaml"
                | ".python-version"
                | "CHANGELOG.md"
                | "CHANGELOG.md.j2"
                | "CONTRIBUTING.md"
                | "Cargo.lock"
                | "Cargo.toml"
                | "LICENSE"
                | "README.md"
                | "SECURITY.md"
                | "clippy.toml"
                | "cspell.json"
                | "deny.toml"
                | "package-lock.json"
                | "package.json"
                | "pyproject.toml"
                | "rust-toolchain.toml"
                | "rustfmt.toml"
                | "uv.lock"
                | ".github/CODEOWNERS"
                | ".github/SUPPORT.md"
                | ".github/copilot-instructions.md"
                | ".github/dependabot.yml"
                | ".github/dependency-review-config.yml"
                | ".github/dev-tools.toml"
                | ".github/labels.toml"
                | ".github/pull_request_template.md"
                | ".github/release.yml"
        )
        || is_known_workflow_or_action(path)
}

fn is_known_crate_path(path: &str) -> bool {
    const CRATES: &[&str] = &[
        "tq-cli",
        "tq-config",
        "tq-core",
        "tq-dev",
        "tq-discovery",
        "tq-docsgen",
        "tq-engine",
        "tq-release",
        "tq-reporting",
        "tq-rules",
    ];

    path.strip_prefix("crates/")
        .and_then(|path| path.split('/').next())
        .is_some_and(|crate_name| CRATES.contains(&crate_name))
}

fn is_known_workflow_or_action(path: &str) -> bool {
    const WORKFLOWS: &[&str] = &[
        "ci.yml",
        "codeql.yml",
        "copilot-setup-steps.yml",
        "dependency-review.yml",
        "docs-external-links.yml",
        "docs-pages.yml",
        "docs-security.yml",
        "frozen-pre-commit-policy.yml",
        "pinned-actions-policy.yml",
        "pinned-external-dependency-drift.yml",
        "publish.yml",
        "rust-maintenance-tool-pins.yml",
        "rust-security-advisories.yml",
        "stale-dependencies.yml",
    ];
    const ACTIONS: &[&str] = &[
        "setup-docs/action.yml",
        "setup-node/action.yml",
        "setup-python-uv/action.yml",
        "setup-rust/action.yml",
        "setup-rust-maintenance-tools/action.yml",
        "setup-rust-security-tools/action.yml",
        "sync-drift-issue/action.yml",
    ];

    path.strip_prefix(".github/workflows/")
        .is_some_and(|name| WORKFLOWS.contains(&name))
        || path
            .strip_prefix(".github/actions/")
            .is_some_and(|name| ACTIONS.contains(&name))
}

fn git_changed_paths(
    repo_root: &Path,
    base_ref: &str,
    head_ref: &str,
) -> Result<Vec<ChangedPath>, DevError> {
    let args = ["diff", "--name-only", "-z", base_ref, head_ref];
    let output = git_output(repo_root, &args)?;
    if !output.status.success() {
        return Err(DevError::Git {
            args: args.join(" "),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        });
    }

    parse_nul_paths(&output.stdout)
}

fn parse_nul_paths(output: &[u8]) -> Result<Vec<ChangedPath>, DevError> {
    output
        .split(|byte| *byte == 0)
        .filter(|bytes| !bytes.is_empty())
        .map(|bytes| {
            let value =
                std::str::from_utf8(bytes).map_err(|source| DevError::CommandOutputParse {
                    program: "git".to_owned(),
                    message: format!("changed path is not UTF-8: {source}"),
                })?;
            ChangedPath::parse(value)
        })
        .collect()
}

fn try_git_stdout(repo_root: &Path, args: &[&str]) -> Result<Option<String>, DevError> {
    let output = git_output(repo_root, args)?;
    if !output.status.success() {
        return Ok(None);
    }
    Ok(Some(String::from_utf8_lossy(&output.stdout).into_owned()))
}

fn git_output(repo_root: &Path, args: &[&str]) -> Result<std::process::Output, DevError> {
    Command::new("git")
        .current_dir(repo_root)
        .args(args)
        .output()
        .map_err(|source| DevError::GitIo {
            args: args.join(" "),
            source,
        })
}
