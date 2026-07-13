use std::fs;
use std::path::Path;
use std::process::Command;

use tq_dev::error::DevError;
use tq_dev::external_pins::{verify_action_pins, verify_pre_commit_pins};

const SHA: &str = "9c091bb21b7c1c1d1991bb908d89e4e9dddfe3e0"; // pragma: allowlist secret

fn write(path: &Path, contents: &str) {
    fs::create_dir_all(path.parent().expect("fixture path must have a parent"))
        .expect("create fixture directory");
    fs::write(path, contents).expect("write fixture");
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

fn init_repo(root: &Path) {
    git(root, &["init", "--initial-branch", "main"]);
}

#[test]
fn action_policy_accepts_sha_pins_and_explicit_local_kinds() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();
    init_repo(root);
    write(
        &root.join(".github/workflows/ci.yml"),
        &format!(
            "steps:\n  - uses: actions/checkout@{SHA}\n  - uses: ./local\n  - uses: docker://alpine@sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n"
        ),
    );
    git(root, &["add", "--all"]);

    verify_action_pins(root).expect("all action references should be frozen");
}

#[test]
fn action_policy_rejects_mutable_docker_tags() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();
    init_repo(root);
    write(
        &root.join(".github/workflows/ci.yml"),
        "steps:\n  - uses: docker://alpine:3.20\n",
    );
    git(root, &["add", "--all"]);

    let error = verify_action_pins(root).expect_err("mutable Docker tag must fail policy");
    assert!(error.to_string().contains("immutable sha256 digest"));
}

#[test]
fn action_policy_rejects_missing_non_sha_and_malformed_refs() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();
    init_repo(root);
    let workflow = root.join(".github/workflows/ci.yml");
    write(
        &workflow,
        "steps:\n  - uses: actions/checkout@v4\n  - uses: owner/action\n",
    );
    git(root, &["add", "--all"]);

    let error = verify_action_pins(root).expect_err("mutable action refs must fail policy");
    let DevError::PolicyViolation { details } = error else {
        panic!("expected policy violation");
    };
    assert!(details.contains("actions/checkout"));
    assert!(details.contains("owner/action is missing a pinned revision"));

    write(&workflow, "steps:\n  - uses:\n");
    let error = verify_action_pins(root).expect_err("empty uses must be malformed input");
    assert!(matches!(error, DevError::InvalidInput { .. }));
}

#[test]
fn pre_commit_policy_rejects_missing_and_non_sha_revisions() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();
    write(
        &root.join(".pre-commit-config.yaml"),
        concat!(
            "repos:\n",
            "  - repo: local\n",
            "    hooks: []\n",
            "  - repo: https://github.com/example/missing\n",
            "    hooks: []\n",
            "  - repo: https://github.com/example/tagged\n",
            "    rev: v1.2.3\n",
            "    hooks: []\n",
        ),
    );

    let error = verify_pre_commit_pins(root).expect_err("mutable revisions must fail policy");
    let DevError::PolicyViolation { details } = error else {
        panic!("expected policy violation");
    };
    assert!(details.contains("example/missing is missing a pinned revision"));
    assert!(details.contains("example/tagged"));
}

#[test]
fn pre_commit_policy_accepts_external_sha_and_local_repository() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();
    write(
        &root.join(".pre-commit-config.yaml"),
        &format!(
            "repos:\n  - repo: local\n    hooks: []\n  - repo: https://github.com/example/frozen\n    rev: {SHA}\n    hooks: []\n"
        ),
    );

    verify_pre_commit_pins(root).expect("external repository should be frozen");
}

#[test]
fn policy_workflows_delegate_yaml_parsing_to_tq_dev() {
    let actions = include_str!("../../../.github/workflows/pinned-actions-policy.yml");
    let pre_commit = include_str!("../../../.github/workflows/frozen-pre-commit-policy.yml");

    assert!(actions.contains("cargo dev policy verify-action-pins"));
    assert!(!actions.contains("sed -E"));
    assert!(!actions.contains("grep -nE"));
    assert!(pre_commit.contains("cargo dev policy verify-pre-commit-pins"));
    assert!(!pre_commit.contains("ruby"));
    assert!(!pre_commit.contains("YAML.safe_load"));
}
