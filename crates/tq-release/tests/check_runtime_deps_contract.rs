use std::fs;
use std::path::Path;
use std::process::Command;

use tq_release::RuntimeDependencyChange;

/// Minimal two-crate workspace whose shipped root crate is `tq-cli`.
struct Workspace {
    serde_version: &'static str,
    cli_serde_dependency: &'static str,
    tooling_marker: &'static str,
}

impl Workspace {
    const fn baseline() -> Self {
        Self {
            serde_version: "1.0.200",
            cli_serde_dependency: "serde = { workspace = true }",
            tooling_marker: "initial",
        }
    }

    fn write(&self, root: &Path) {
        write(
            &root.join("Cargo.toml"),
            &format!(
                concat!(
                    "[workspace]\n",
                    "members = [\"crates/tq-cli\", \"crates/tq-core\"]\n\n",
                    "[workspace.dependencies]\n",
                    "serde = \"{serde}\"\n",
                    "tq-core = {{ path = \"crates/tq-core\" }}\n",
                ),
                serde = self.serde_version,
            ),
        );
        write(
            &root.join("crates/tq-cli/Cargo.toml"),
            &format!(
                concat!(
                    "[package]\n",
                    "name = \"tq-cli\"\n\n",
                    "[dependencies]\n",
                    "{serde_dependency}\n",
                    "tq-core = {{ workspace = true }}\n",
                ),
                serde_dependency = self.cli_serde_dependency,
            ),
        );
        write(
            &root.join("crates/tq-core/Cargo.toml"),
            concat!(
                "[package]\n",
                "name = \"tq-core\"\n\n",
                "[dependencies]\n",
                "serde = { workspace = true }\n",
            ),
        );
        write(
            &root.join("Cargo.lock"),
            &format!(
                concat!(
                    "version = 4\n\n",
                    "[[package]]\n",
                    "name = \"serde\"\n",
                    "version = \"{serde}\"\n",
                    "source = \"registry+https://github.com/rust-lang/crates.io-index\"\n\n",
                    "[[package]]\n",
                    "name = \"tq-cli\"\n",
                    "version = \"0.1.0\"\n",
                    "dependencies = [\n \"serde\",\n \"tq-core\",\n]\n\n",
                    "[[package]]\n",
                    "name = \"tq-core\"\n",
                    "version = \"0.1.0\"\n",
                    "dependencies = [\n \"serde\",\n]\n",
                ),
                serde = self.serde_version,
            ),
        );
        write(&root.join("tooling.txt"), self.tooling_marker);
    }
}

fn write(path: &Path, contents: &str) {
    fs::create_dir_all(path.parent().expect("parent path must exist"))
        .expect("create parent directories");
    fs::write(path, contents).expect("write file");
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

fn commit_all(repo_root: &Path, message: &str) -> String {
    git(repo_root, &["add", "--all"]);
    git(repo_root, &["commit", "--message", message]);
    let output = Command::new("git")
        .current_dir(repo_root)
        .args(["rev-parse", "HEAD"])
        .output()
        .expect("run git rev-parse");
    assert!(output.status.success(), "git rev-parse HEAD failed");
    String::from_utf8_lossy(&output.stdout).trim().to_owned()
}

fn init_repo(root: &Path) {
    git(root, &["init", "--initial-branch", "main"]);
    git(root, &["config", "user.email", "ci@example.com"]);
    git(root, &["config", "user.name", "CI"]);
    git(root, &["config", "commit.gpgsign", "false"]);
}

#[test]
fn reports_unchanged_when_only_tooling_files_change() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();
    init_repo(root);

    Workspace::baseline().write(root);
    let base = commit_all(root, "baseline workspace");

    Workspace {
        tooling_marker: "edited",
        ..Workspace::baseline()
    }
    .write(root);
    let head = commit_all(root, "tooling-only change");

    let change = tq_release::check_runtime_dep_changes(root, &base, &head)
        .expect("runtime dep check should succeed");

    assert_eq!(change, RuntimeDependencyChange::Unchanged);
}

#[test]
fn reports_changed_when_shipped_runtime_dependency_version_changes() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();
    init_repo(root);

    Workspace::baseline().write(root);
    let base = commit_all(root, "baseline workspace");

    Workspace {
        serde_version: "1.0.210",
        ..Workspace::baseline()
    }
    .write(root);
    let head = commit_all(root, "bump shipped runtime dependency");

    let change = tq_release::check_runtime_dep_changes(root, &base, &head)
        .expect("runtime dep check should succeed");

    assert_eq!(change, RuntimeDependencyChange::Changed);
}

#[test]
fn reports_changed_when_member_enables_dependency_feature_without_lock_change() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();
    init_repo(root);

    Workspace::baseline().write(root);
    let base = commit_all(root, "baseline workspace");

    // Only the member-level feature set changes; the workspace version and the
    // resolved lockfile are identical, so this is caught by the manifest signal.
    Workspace {
        cli_serde_dependency: "serde = { workspace = true, features = [\"derive\"] }",
        ..Workspace::baseline()
    }
    .write(root);
    let head = commit_all(root, "enable shipped dependency feature");

    let change = tq_release::check_runtime_dep_changes(root, &base, &head)
        .expect("runtime dep check should succeed");

    assert_eq!(change, RuntimeDependencyChange::Changed);
}

#[test]
fn reports_unchanged_for_identical_refs() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();
    init_repo(root);

    Workspace::baseline().write(root);
    let base = commit_all(root, "baseline workspace");

    let change = tq_release::check_runtime_dep_changes(root, &base, &base)
        .expect("runtime dep check should succeed");

    assert_eq!(change, RuntimeDependencyChange::Unchanged);
}
