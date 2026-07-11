use std::fs;
use std::path::Path;

fn write(path: &Path, contents: &str) {
    fs::create_dir_all(path.parent().expect("parent path must exist"))
        .expect("create parent directories");
    fs::write(path, contents).expect("write file");
}

const MEMBER_MANIFEST: &str = concat!(
    "[package]\n",
    "name = \"tq-core\"\n",
    "version.workspace = true\n",
    "publish.workspace = true\n",
);

fn write_consistent_workspace(root: &Path) {
    write(
        &root.join("Cargo.toml"),
        concat!(
            "[workspace]\n",
            "members = [\"crates/tq-core\", \"crates/tq-engine\"]\n",
            "\n",
            "[workspace.package]\n",
            "version = \"0.7.0\"\n",
            "publish = false\n",
            "\n",
            "[workspace.dependencies]\n",
            "tq-core = { path = \"crates/tq-core\" }\n",
            "tq-engine = { path = \"crates/tq-engine\" }\n",
        ),
    );
    write(
        &root.join("CHANGELOG.md"),
        concat!(
            "# Changelog\n\n",
            "## [0.7.0] - 2026-04-06\n\n",
            "### Changed\n\n",
            "- Example\n",
        ),
    );
    write(&root.join("crates/tq-core/Cargo.toml"), MEMBER_MANIFEST);
    write(
        &root.join("crates/tq-engine/Cargo.toml"),
        concat!(
            "[package]\n",
            "name = \"tq-engine\"\n",
            "version.workspace = true\n",
            "publish.workspace = true\n",
        ),
    );
}

#[test]
fn verify_workspace_version_passes_for_consistent_repo_shape() {
    let temp = tempfile::tempdir().expect("tempdir");
    write_consistent_workspace(temp.path());

    tq_dev::workspace_version::verify_workspace_version(temp.path())
        .expect("workspace version should pass");
}

#[test]
fn verify_workspace_version_rejects_mismatched_internal_dependency_path() {
    let temp = tempfile::tempdir().expect("tempdir");
    write_consistent_workspace(temp.path());
    write(
        &temp.path().join("Cargo.toml"),
        concat!(
            "[workspace]\n",
            "members = [\"crates/tq-core\"]\n",
            "\n",
            "[workspace.package]\n",
            "version = \"0.7.0\"\n",
            "publish = false\n",
            "\n",
            "[workspace.dependencies]\n",
            "tq-core = { path = \"crates/elsewhere\" }\n",
        ),
    );

    let error = tq_dev::workspace_version::verify_workspace_version(temp.path())
        .expect_err("mismatched internal dependency path should fail");

    assert!(
        error
            .to_string()
            .contains("workspace.dependencies.tq-core.path")
    );
}

#[test]
fn verify_workspace_version_rejects_missing_current_changelog_heading() {
    let temp = tempfile::tempdir().expect("tempdir");
    write_consistent_workspace(temp.path());
    write(
        &temp.path().join("CHANGELOG.md"),
        "# Changelog\n\n## [0.6.3] - 2026-03-04\n",
    );

    let error = tq_dev::workspace_version::verify_workspace_version(temp.path())
        .expect_err("missing changelog heading should fail");

    assert!(
        error
            .to_string()
            .contains("CHANGELOG.md top release heading")
    );
}

#[test]
fn verify_workspace_version_rejects_member_without_workspace_version_inheritance() {
    let temp = tempfile::tempdir().expect("tempdir");
    write_consistent_workspace(temp.path());
    write(
        &temp.path().join("crates/tq-core/Cargo.toml"),
        concat!(
            "[package]\n",
            "name = \"tq-core\"\n",
            "version = \"0.7.0\"\n",
            "publish.workspace = true\n",
        ),
    );

    let error = tq_dev::workspace_version::verify_workspace_version(temp.path())
        .expect_err("members must inherit workspace version");

    assert!(error.to_string().contains("must inherit package.version"));
}

#[test]
fn verify_workspace_version_rejects_member_without_workspace_publish_inheritance() {
    let temp = tempfile::tempdir().expect("tempdir");
    write_consistent_workspace(temp.path());
    write(
        &temp.path().join("crates/tq-core/Cargo.toml"),
        concat!(
            "[package]\n",
            "name = \"tq-core\"\n",
            "version.workspace = true\n",
        ),
    );

    let error = tq_dev::workspace_version::verify_workspace_version(temp.path())
        .expect_err("members must inherit workspace publish policy");

    assert!(error.to_string().contains("must inherit package.publish"));
}

#[test]
fn verify_workspace_version_fails_with_manifest_path_on_invalid_toml() {
    let temp = tempfile::tempdir().expect("tempdir");

    write(&temp.path().join("Cargo.toml"), "not valid toml\n");
    write(&temp.path().join("CHANGELOG.md"), "# Changelog\n");

    let error = tq_dev::workspace_version::verify_workspace_version(temp.path())
        .expect_err("invalid workspace manifest should fail");

    let message = error.to_string();
    assert!(message.contains("invalid input"));
    assert!(message.contains("Cargo.toml"));
}
