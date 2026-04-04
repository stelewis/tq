use std::fs;
use std::path::Path;

fn write(path: &Path, contents: &str) {
    fs::create_dir_all(path.parent().expect("parent path must exist"))
        .expect("create parent directories");
    fs::write(path, contents).expect("write file");
}

#[test]
fn verify_workspace_version_passes_for_consistent_repo_shape() {
    let temp = tempfile::tempdir().expect("tempdir");

    write(
        &temp.path().join("Cargo.toml"),
        concat!(
            "[workspace]\n",
            "members = [\"crates/tq-core\", \"crates/tq-engine\"]\n",
            "\n",
            "[workspace.package]\n",
            "version = \"0.9.0\"\n",
            "\n",
            "[workspace.dependencies]\n",
            "tq-core = { version = \"0.9.0\", path = \"crates/tq-core\" }\n",
            "tq-engine = { version = \"0.9.0\", path = \"crates/tq-engine\" }\n",
        ),
    );
    write(
        &temp.path().join("CHANGELOG.md"),
        concat!(
            "# Changelog\n\n",
            "## [0.9.0] - 2026-04-05\n\n",
            "### Changed\n\n",
            "- Example\n",
        ),
    );
    write(
        &temp.path().join("crates/tq-core/Cargo.toml"),
        concat!(
            "[package]\n",
            "name = \"tq-core\"\n",
            "version.workspace = true\n",
        ),
    );
    write(
        &temp.path().join("crates/tq-engine/Cargo.toml"),
        concat!(
            "[package]\n",
            "name = \"tq-engine\"\n",
            "version.workspace = true\n",
        ),
    );

    tq_release::verify_workspace_version(temp.path()).expect("workspace version should pass");
}

#[test]
fn verify_workspace_version_rejects_mismatched_internal_dependency_version() {
    let temp = tempfile::tempdir().expect("tempdir");

    write(
        &temp.path().join("Cargo.toml"),
        concat!(
            "[workspace]\n",
            "members = [\"crates/tq-core\"]\n",
            "\n",
            "[workspace.package]\n",
            "version = \"0.9.0\"\n",
            "\n",
            "[workspace.dependencies]\n",
            "tq-core = { version = \"0.8.0\", path = \"crates/tq-core\" }\n",
        ),
    );
    write(
        &temp.path().join("CHANGELOG.md"),
        "# Changelog\n\n## [0.9.0] - 2026-04-05\n",
    );
    write(
        &temp.path().join("crates/tq-core/Cargo.toml"),
        concat!(
            "[package]\n",
            "name = \"tq-core\"\n",
            "version.workspace = true\n",
        ),
    );

    let error = tq_release::verify_workspace_version(temp.path())
        .expect_err("mismatched internal dependency version should fail");

    assert!(
        error
            .to_string()
            .contains("workspace.dependencies.tq-core.version")
    );
}

#[test]
fn verify_workspace_version_rejects_missing_current_changelog_heading() {
    let temp = tempfile::tempdir().expect("tempdir");

    write(
        &temp.path().join("Cargo.toml"),
        concat!(
            "[workspace]\n",
            "members = [\"crates/tq-core\"]\n",
            "\n",
            "[workspace.package]\n",
            "version = \"0.9.0\"\n",
            "\n",
            "[workspace.dependencies]\n",
            "tq-core = { version = \"0.9.0\", path = \"crates/tq-core\" }\n",
        ),
    );
    write(
        &temp.path().join("CHANGELOG.md"),
        "# Changelog\n\n## [0.8.0] - 2026-04-04\n",
    );
    write(
        &temp.path().join("crates/tq-core/Cargo.toml"),
        concat!(
            "[package]\n",
            "name = \"tq-core\"\n",
            "version.workspace = true\n",
        ),
    );

    let error = tq_release::verify_workspace_version(temp.path())
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

    write(
        &temp.path().join("Cargo.toml"),
        concat!(
            "[workspace]\n",
            "members = [\"crates/tq-core\"]\n",
            "\n",
            "[workspace.package]\n",
            "version = \"0.9.0\"\n",
            "\n",
            "[workspace.dependencies]\n",
            "tq-core = { version = \"0.9.0\", path = \"crates/tq-core\" }\n",
        ),
    );
    write(
        &temp.path().join("CHANGELOG.md"),
        "# Changelog\n\n## [0.9.0] - 2026-04-05\n",
    );
    write(
        &temp.path().join("crates/tq-core/Cargo.toml"),
        concat!(
            "[package]\n",
            "name = \"tq-core\"\n",
            "version = \"0.9.0\"\n",
        ),
    );

    let error = tq_release::verify_workspace_version(temp.path())
        .expect_err("members must inherit workspace version");

    assert!(error.to_string().contains("must inherit package.version"));
}
