use std::fs;
use std::path::Path;
use std::process::Command;

use assert_cmd::Command as AssertCommand;
use assert_cmd::assert::OutputAssertExt;

fn write(path: &Path, contents: &str) {
    fs::create_dir_all(path.parent().expect("parent path must exist"))
        .expect("create parent directories");
    fs::write(path, contents).expect("write file");
}

fn git(repo_root: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .current_dir(repo_root)
        .args(args)
        .output()
        .expect("run git command");
    assert!(
        output.status.success(),
        "git command failed: {}\n{}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("git stdout should be utf8")
}

fn commit_all(repo_root: &Path, message: &str) -> String {
    git(repo_root, &["add", "."]);
    git(
        repo_root,
        &["-c", "commit.gpgsign=false", "commit", "-m", message],
    );
    git(repo_root, &["rev-parse", "HEAD"]).trim().to_owned()
}

fn init_repo(repo_root: &Path) {
    git(repo_root, &["init"]);
    git(repo_root, &["config", "user.name", "Test User"]);
    git(repo_root, &["config", "user.email", "test@example.com"]);
}

fn root_manifest(root_license: &str) -> String {
    concat!(
        "[workspace]\n",
        "members = [\n",
        "  \"crates/tq-cli\",\n",
        "  \"crates/tq-config\",\n",
        "  \"crates/tq-core\",\n",
        "  \"crates/tq-discovery\",\n",
        "  \"crates/tq-engine\",\n",
        "  \"crates/tq-reporting\",\n",
        "  \"crates/tq-rules\",\n",
        "]\n",
        "resolver = \"3\"\n",
        "\n",
        "[workspace.package]\n",
        "version = \"0.11.0\"\n",
        "edition = \"2024\"\n",
        "license = \"__LICENSE__\"\n",
        "\n",
        "[workspace.dependencies]\n",
        "clap = { version = \"4.6.0\", features = [\"derive\"] }\n",
        "serde = { version = \"1.0.0\", features = [\"derive\"] }\n",
        "serde_json = \"1.0.0\"\n",
        "thiserror = \"2.0.18\"\n",
        "toml = \"1.0.0\"\n",
        "tq-cli = { version = \"0.11.0\", path = \"crates/tq-cli\" }\n",
        "tq-config = { version = \"0.11.0\", path = \"crates/tq-config\" }\n",
        "tq-core = { version = \"0.11.0\", path = \"crates/tq-core\" }\n",
        "tq-discovery = { version = \"0.11.0\", path = \"crates/tq-discovery\" }\n",
        "tq-engine = { version = \"0.11.0\", path = \"crates/tq-engine\" }\n",
        "tq-reporting = { version = \"0.11.0\", path = \"crates/tq-reporting\" }\n",
        "tq-rules = { version = \"0.11.0\", path = \"crates/tq-rules\" }\n",
    )
    .replace("__LICENSE__", root_license)
}

fn write_member_manifests(repo_root: &Path) {
    write(
        &repo_root.join("crates/tq-cli/Cargo.toml"),
        concat!(
            "[package]\n",
            "name = \"tq-cli\"\n",
            "version.workspace = true\n",
            "\n",
            "[dependencies]\n",
            "clap.workspace = true\n",
            "thiserror.workspace = true\n",
            "tq-config.workspace = true\n",
            "tq-core.workspace = true\n",
            "tq-engine.workspace = true\n",
            "tq-reporting.workspace = true\n",
            "tq-rules.workspace = true\n",
            "\n",
            "[dev-dependencies]\n",
            "assert_cmd = \"2.2.0\"\n",
        ),
    );
    write(
        &repo_root.join("crates/tq-config/Cargo.toml"),
        concat!(
            "[package]\n",
            "name = \"tq-config\"\n",
            "version.workspace = true\n",
            "\n",
            "[dependencies]\n",
            "serde.workspace = true\n",
            "thiserror.workspace = true\n",
            "toml.workspace = true\n",
            "tq-core.workspace = true\n",
        ),
    );
    write(
        &repo_root.join("crates/tq-core/Cargo.toml"),
        concat!(
            "[package]\n",
            "name = \"tq-core\"\n",
            "version.workspace = true\n",
            "\n",
            "[dependencies]\n",
            "thiserror.workspace = true\n",
        ),
    );
    write(
        &repo_root.join("crates/tq-discovery/Cargo.toml"),
        concat!(
            "[package]\n",
            "name = \"tq-discovery\"\n",
            "version.workspace = true\n",
            "\n",
            "[dependencies]\n",
            "thiserror.workspace = true\n",
        ),
    );
    write(
        &repo_root.join("crates/tq-engine/Cargo.toml"),
        concat!(
            "[package]\n",
            "name = \"tq-engine\"\n",
            "version.workspace = true\n",
            "\n",
            "[dependencies]\n",
            "thiserror.workspace = true\n",
            "tq-core.workspace = true\n",
            "tq-discovery.workspace = true\n",
        ),
    );
    write(
        &repo_root.join("crates/tq-reporting/Cargo.toml"),
        concat!(
            "[package]\n",
            "name = \"tq-reporting\"\n",
            "version.workspace = true\n",
            "\n",
            "[dependencies]\n",
            "serde.workspace = true\n",
            "serde_json.workspace = true\n",
            "thiserror.workspace = true\n",
            "tq-core.workspace = true\n",
            "tq-engine.workspace = true\n",
        ),
    );
    write(
        &repo_root.join("crates/tq-rules/Cargo.toml"),
        concat!(
            "[package]\n",
            "name = \"tq-rules\"\n",
            "version.workspace = true\n",
            "\n",
            "[dependencies]\n",
            "thiserror.workspace = true\n",
            "tq-core.workspace = true\n",
            "tq-engine.workspace = true\n",
        ),
    );
}

fn write_source_files(repo_root: &Path) {
    write(
        &repo_root.join("crates/tq-cli/src/main.rs"),
        "fn main() {}\n",
    );
    write(
        &repo_root.join("crates/tq-config/src/lib.rs"),
        "pub fn config() {}\n",
    );
    write(
        &repo_root.join("crates/tq-core/src/lib.rs"),
        "pub fn core() {}\n",
    );
    write(
        &repo_root.join("crates/tq-discovery/src/lib.rs"),
        "pub fn discovery() {}\n",
    );
    write(
        &repo_root.join("crates/tq-engine/src/lib.rs"),
        "pub fn engine() {}\n",
    );
    write(
        &repo_root.join("crates/tq-reporting/src/lib.rs"),
        "pub fn reporting() {}\n",
    );
    write(
        &repo_root.join("crates/tq-rules/src/lib.rs"),
        "pub fn rules() {}\n",
    );
}

fn write_workspace(repo_root: &Path, root_license: &str, lock_contents: &str) {
    write(&repo_root.join("Cargo.toml"), &root_manifest(root_license));
    write(
        &repo_root.join("CHANGELOG.md"),
        "# Changelog\n\n## [0.11.0] - 2026-04-26\n\n- Example\n",
    );
    write_member_manifests(repo_root);
    write_source_files(repo_root);
    write(&repo_root.join("Cargo.lock"), lock_contents);
}

fn workspace_lock(assert_cmd_version: &str, thiserror_version: &str) -> String {
    workspace_lock_with_thiserror_source(
        assert_cmd_version,
        thiserror_version,
        "registry+https://github.com/rust-lang/crates.io-index",
    )
}

fn workspace_lock_with_thiserror_source(
    assert_cmd_version: &str,
    thiserror_version: &str,
    thiserror_source: &str,
) -> String {
    format!(
        concat!(
            "version = 4\n\n",
            "[[package]]\n",
            "name = \"assert_cmd\"\n",
            "version = \"{}\"\n",
            "source = \"registry+https://github.com/rust-lang/crates.io-index\"\n\n",
            "[[package]]\n",
            "name = \"clap\"\n",
            "version = \"4.6.0\"\n",
            "source = \"registry+https://github.com/rust-lang/crates.io-index\"\n\n",
            "[[package]]\n",
            "name = \"serde\"\n",
            "version = \"1.0.0\"\n",
            "source = \"registry+https://github.com/rust-lang/crates.io-index\"\n\n",
            "[[package]]\n",
            "name = \"serde_json\"\n",
            "version = \"1.0.0\"\n",
            "source = \"registry+https://github.com/rust-lang/crates.io-index\"\n",
            "dependencies = [\n",
            " \"serde\",\n",
            "]\n\n",
            "[[package]]\n",
            "name = \"thiserror\"\n",
            "version = \"{}\"\n",
            "source = \"{}\"\n\n",
            "[[package]]\n",
            "name = \"toml\"\n",
            "version = \"1.0.0\"\n",
            "source = \"registry+https://github.com/rust-lang/crates.io-index\"\n",
            "dependencies = [\n",
            " \"serde\",\n",
            "]\n\n",
            "[[package]]\n",
            "name = \"tq-cli\"\n",
            "version = \"0.11.0\"\n",
            "dependencies = [\n",
            " \"assert_cmd\",\n",
            " \"clap\",\n",
            " \"thiserror\",\n",
            " \"tq-config\",\n",
            " \"tq-core\",\n",
            " \"tq-engine\",\n",
            " \"tq-reporting\",\n",
            " \"tq-rules\",\n",
            "]\n\n",
            "[[package]]\n",
            "name = \"tq-config\"\n",
            "version = \"0.11.0\"\n",
            "dependencies = [\n",
            " \"serde\",\n",
            " \"thiserror\",\n",
            " \"toml\",\n",
            " \"tq-core\",\n",
            "]\n\n",
            "[[package]]\n",
            "name = \"tq-core\"\n",
            "version = \"0.11.0\"\n",
            "dependencies = [\n",
            " \"thiserror\",\n",
            "]\n\n",
            "[[package]]\n",
            "name = \"tq-discovery\"\n",
            "version = \"0.11.0\"\n",
            "dependencies = [\n",
            " \"thiserror\",\n",
            "]\n\n",
            "[[package]]\n",
            "name = \"tq-engine\"\n",
            "version = \"0.11.0\"\n",
            "dependencies = [\n",
            " \"thiserror\",\n",
            " \"tq-core\",\n",
            " \"tq-discovery\",\n",
            "]\n\n",
            "[[package]]\n",
            "name = \"tq-reporting\"\n",
            "version = \"0.11.0\"\n",
            "dependencies = [\n",
            " \"serde\",\n",
            " \"serde_json\",\n",
            " \"thiserror\",\n",
            " \"tq-core\",\n",
            " \"tq-engine\",\n",
            "]\n\n",
            "[[package]]\n",
            "name = \"tq-rules\"\n",
            "version = \"0.11.0\"\n",
            "dependencies = [\n",
            " \"thiserror\",\n",
            " \"tq-core\",\n",
            " \"tq-engine\",\n",
            "]\n",
        ),
        assert_cmd_version, thiserror_version, thiserror_source,
    )
}

fn create_repo() -> tempfile::TempDir {
    let temp = tempfile::tempdir().expect("tempdir");
    init_repo(temp.path());
    write_workspace(temp.path(), "MIT", &workspace_lock("2.2.0", "2.0.18"));
    temp
}

#[test]
fn verify_pr_release_intent_allows_release_none_for_metadata_only_root_manifest_change() {
    let repo = create_repo();
    let base = commit_all(repo.path(), "base");

    write_workspace(
        repo.path(),
        "Apache-2.0",
        &workspace_lock("2.2.0", "2.0.18"),
    );
    let head = commit_all(repo.path(), "metadata-only change");

    Command::new(env!("CARGO_BIN_EXE_tq-release"))
        .current_dir(repo.path())
        .arg("verify-pr-release-intent")
        .arg("--repo-root")
        .arg(repo.path())
        .arg("--base-ref")
        .arg(base)
        .arg("--head-ref")
        .arg(head)
        .arg("--label")
        .arg("release:none")
        .assert()
        .success();
}

#[test]
fn verify_pr_release_intent_rejects_release_none_for_lock_only_runtime_dependency_change() {
    let repo = create_repo();
    let base = commit_all(repo.path(), "base");

    write_workspace(repo.path(), "MIT", &workspace_lock("2.2.0", "2.0.19"));
    let head = commit_all(repo.path(), "runtime lock change");

    let assert = AssertCommand::new(env!("CARGO_BIN_EXE_tq-release"))
        .current_dir(repo.path())
        .arg("verify-pr-release-intent")
        .arg("--repo-root")
        .arg(repo.path())
        .arg("--base-ref")
        .arg(base)
        .arg("--head-ref")
        .arg(head)
        .arg("--label")
        .arg("release:none")
        .assert();

    let output = assert.get_output();
    assert_eq!(output.status.code(), Some(1));
    assert!(
        String::from_utf8(output.stderr.clone())
            .expect("stderr should be utf8")
            .contains("shipped runtime dependency change")
    );
}

#[test]
fn verify_pr_release_intent_rejects_release_none_for_runtime_dependency_source_change() {
    let repo = create_repo();
    let base = commit_all(repo.path(), "base");

    write_workspace(
        repo.path(),
        "MIT",
        &workspace_lock_with_thiserror_source(
            "2.2.0",
            "2.0.18",
            "git+https://example.invalid/thiserror?rev=deadbeef#deadbeef",
        ),
    );
    let head = commit_all(repo.path(), "runtime source change");

    let assert = AssertCommand::new(env!("CARGO_BIN_EXE_tq-release"))
        .current_dir(repo.path())
        .arg("verify-pr-release-intent")
        .arg("--repo-root")
        .arg(repo.path())
        .arg("--base-ref")
        .arg(base)
        .arg("--head-ref")
        .arg(head)
        .arg("--label")
        .arg("release:none")
        .assert();

    let output = assert.get_output();
    assert_eq!(output.status.code(), Some(1));
    assert!(
        String::from_utf8(output.stderr.clone())
            .expect("stderr should be utf8")
            .contains("shipped runtime dependency change")
    );
}

#[test]
fn verify_pr_release_intent_allows_release_none_for_lock_only_dev_dependency_change() {
    let repo = create_repo();
    let base = commit_all(repo.path(), "base");

    write_workspace(repo.path(), "MIT", &workspace_lock("2.3.0", "2.0.18"));
    let head = commit_all(repo.path(), "dev lock change");

    Command::new(env!("CARGO_BIN_EXE_tq-release"))
        .current_dir(repo.path())
        .arg("verify-pr-release-intent")
        .arg("--repo-root")
        .arg(repo.path())
        .arg("--base-ref")
        .arg(base)
        .arg("--head-ref")
        .arg(head)
        .arg("--label")
        .arg("release:none")
        .assert()
        .success();
}
