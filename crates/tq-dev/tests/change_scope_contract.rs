use std::fs;
use std::path::Path;
use std::process::Command;

use tq_dev::change_scope::{ChangeSource, ChangedPath, classify_git_changes, classify_paths};

fn changed_path(value: &str) -> ChangedPath {
    ChangedPath::parse(value).expect("fixture path should be valid")
}

fn git(repo_root: &Path, args: &[&str]) {
    let output = Command::new("git")
        .current_dir(repo_root)
        .args(args)
        .output()
        .expect("run git");
    assert!(
        output.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr),
    );
}

fn commit(repo_root: &Path, message: &str) -> String {
    git(repo_root, &["add", "--all"]);
    git(repo_root, &["commit", "--message", message]);
    let output = Command::new("git")
        .current_dir(repo_root)
        .args(["rev-parse", "HEAD"])
        .output()
        .expect("run git rev-parse");
    assert!(output.status.success());
    String::from_utf8_lossy(&output.stdout).trim().to_owned()
}

#[test]
fn classifies_docs_only_and_scoped_policy_inputs() {
    let docs = classify_paths([
        changed_path("docs/guide/quickstart.md"),
        changed_path("README.md"),
    ])
    .expect("paths should classify");

    assert!(docs.docs_only);
    assert!(docs.docs_sync.should_run());
    assert!(docs.docs_build.should_run());
    assert!(!docs.rust_security.should_run());
    assert!(!docs.runtime_dependency_inputs.should_run());
    assert!(!docs.broad_gate.should_run());

    let dependencies = classify_paths([
        changed_path("Cargo.lock"),
        changed_path("crates/tq-engine/Cargo.toml"),
    ])
    .expect("paths should classify");

    assert!(!dependencies.docs_only);
    assert!(dependencies.runtime_dependency_inputs.should_run());
    assert!(dependencies.rust_security.should_run());
    assert!(dependencies.release_relevant.should_run());
    assert!(!dependencies.broad_gate.should_run());
}

#[test]
fn unknown_paths_enable_every_safety_gate() {
    let unknown = changed_path("new-automation/policy.toml");
    let scope = classify_paths([unknown.clone()]).expect("path should classify");

    assert!(scope.broad_gate.should_run());
    assert!(scope.runtime_dependency_inputs.should_run());
    assert!(scope.rust_security.should_run());
    assert!(scope.docs_security.should_run());
    assert!(scope.release_relevant.should_run());
    assert!(scope.docs_sync.should_run());
    assert!(scope.docs_build.should_run());
    assert_eq!(
        scope.unknown_paths.into_iter().collect::<Vec<_>>(),
        vec![unknown]
    );
}

#[test]
fn rejects_paths_that_escape_or_bypass_repository_normalization() {
    for value in ["../Cargo.toml", "/tmp/Cargo.toml", "docs\\guide.md", ""] {
        ChangedPath::parse(value).expect_err("invalid path must be rejected");
    }
}

#[test]
fn classifies_git_ranges_and_broadens_when_merge_base_is_missing() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();
    git(root, &["init", "--initial-branch", "main"]);
    git(root, &["config", "user.email", "ci@example.com"]);
    git(root, &["config", "user.name", "CI"]);
    git(root, &["config", "commit.gpgsign", "false"]);

    fs::write(root.join("README.md"), "first\n").expect("write README");
    let base = commit(root, "base");
    fs::write(root.join("README.md"), "second\n").expect("update README");
    let head = commit(root, "head");

    let scope = classify_git_changes(
        root,
        ChangeSource::PullRequest {
            base_ref: &base,
            head_ref: &head,
        },
    )
    .expect("git range should classify");
    assert!(scope.docs_only);
    assert!(!scope.broad_gate.should_run());

    let missing = classify_git_changes(
        root,
        ChangeSource::PullRequest {
            base_ref: "missing-base",
            head_ref: &head,
        },
    )
    .expect("missing merge base should broaden instead of skip");
    assert!(missing.broad_gate.should_run());
    assert!(missing.rust_security.should_run());
    assert!(missing.docs_security.should_run());
}

#[test]
fn every_tracked_repository_path_has_an_explicit_classification() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("tq-dev must be inside the workspace root");
    let output = Command::new("git")
        .current_dir(repo_root)
        .args(["ls-files", "-z"])
        .output()
        .expect("list tracked files");
    assert!(output.status.success());
    let paths = output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|bytes| !bytes.is_empty())
        .map(|bytes| {
            let value = std::str::from_utf8(bytes).expect("tracked path must be UTF-8");
            ChangedPath::parse(value).expect("tracked path must be normalized")
        })
        .collect::<Vec<_>>();

    let scope = classify_paths(paths).expect("tracked paths should classify");

    assert!(scope.unknown_paths.is_empty());
}

#[test]
fn ci_workflow_delegates_change_classification_to_tq_dev() {
    let workflow = include_str!("../../../.github/workflows/ci.yml");

    assert!(workflow.contains("cargo dev change-scope"));
    assert!(workflow.contains("outputs.runtime_dependency_inputs"));
    assert!(!workflow.contains("changed_files="));
    assert!(!workflow.contains("grep -Eq"));
}
