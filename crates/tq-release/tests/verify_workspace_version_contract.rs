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
            "version = \"0.7.0\"\n",
            "\n",
            "[workspace.dependencies]\n",
            "tq-core = { version = \"0.7.0\", path = \"crates/tq-core\" }\n",
            "tq-engine = { version = \"0.7.0\", path = \"crates/tq-engine\" }\n",
        ),
    );
    write(
        &temp.path().join("CHANGELOG.md"),
        concat!(
            "# Changelog\n\n",
            "## [0.7.0] - 2026-04-06\n\n",
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
            "version = \"0.7.0\"\n",
            "\n",
            "[workspace.dependencies]\n",
            "tq-core = { version = \"0.6.3\", path = \"crates/tq-core\" }\n",
        ),
    );
    write(
        &temp.path().join("CHANGELOG.md"),
        "# Changelog\n\n## [0.7.0] - 2026-04-06\n",
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
            "version = \"0.7.0\"\n",
            "\n",
            "[workspace.dependencies]\n",
            "tq-core = { version = \"0.7.0\", path = \"crates/tq-core\" }\n",
        ),
    );
    write(
        &temp.path().join("CHANGELOG.md"),
        "# Changelog\n\n## [0.6.3] - 2026-03-04\n",
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
            "version = \"0.7.0\"\n",
            "\n",
            "[workspace.dependencies]\n",
            "tq-core = { version = \"0.7.0\", path = \"crates/tq-core\" }\n",
        ),
    );
    write(
        &temp.path().join("CHANGELOG.md"),
        "# Changelog\n\n## [0.7.0] - 2026-04-06\n",
    );
    write(
        &temp.path().join("crates/tq-core/Cargo.toml"),
        concat!(
            "[package]\n",
            "name = \"tq-core\"\n",
            "version = \"0.7.0\"\n",
        ),
    );

    let error = tq_release::verify_workspace_version(temp.path())
        .expect_err("members must inherit workspace version");

    assert!(error.to_string().contains("must inherit package.version"));
}

#[test]
fn verify_workspace_version_uses_workspace_specific_parse_errors() {
    let temp = tempfile::tempdir().expect("tempdir");

    write(&temp.path().join("Cargo.toml"), "not valid toml\n");
    write(&temp.path().join("CHANGELOG.md"), "# Changelog\n");

    let error = tq_release::verify_workspace_version(temp.path())
        .expect_err("invalid workspace manifest should fail");

    assert!(
        error
            .to_string()
            .contains("invalid workspace version input")
    );
    assert!(!error.to_string().contains("invalid Dependabot config"));
}

#[test]
fn sync_workspace_dependency_versions_updates_internal_crate_entries() {
    let temp = tempfile::tempdir().expect("tempdir");

    write(
        &temp.path().join("Cargo.toml"),
        concat!(
            "[workspace]\n",
            "members = [\"crates/tq-core\", \"crates/tq-engine\"]\n",
            "\n",
            "[workspace.package]\n",
            "version = \"0.7.1\"\n",
            "\n",
            "[workspace.dependencies]\n",
            "serde = \"1.0.228\"\n",
            "tq-core = { version = \"0.7.0\", path = \"crates/tq-core\" }\n",
            "tq-engine = { version = \"0.7.0\", path = \"crates/tq-engine\" }\n",
        ),
    );
    write(
        &temp.path().join("CHANGELOG.md"),
        "# Changelog\n\n## [0.7.1] - 2026-04-06\n",
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

    tq_release::sync_workspace_dependency_versions(temp.path())
        .expect("workspace dependency sync should succeed");

    let cargo_toml = fs::read_to_string(temp.path().join("Cargo.toml")).expect("read Cargo.toml");

    assert!(cargo_toml.contains("tq-core = { version = \"0.7.1\", path = \"crates/tq-core\" }"));
    assert!(
        cargo_toml.contains("tq-engine = { version = \"0.7.1\", path = \"crates/tq-engine\" }")
    );
    assert!(cargo_toml.contains("serde = \"1.0.228\""));
}

#[test]
fn sync_workspace_dependency_versions_rejects_missing_internal_entry() {
    let temp = tempfile::tempdir().expect("tempdir");

    write(
        &temp.path().join("Cargo.toml"),
        concat!(
            "[workspace]\n",
            "members = [\"crates/tq-core\"]\n",
            "\n",
            "[workspace.package]\n",
            "version = \"0.7.1\"\n",
            "\n",
            "[workspace.dependencies]\n",
            "serde = \"1.0.228\"\n",
        ),
    );
    write(
        &temp.path().join("CHANGELOG.md"),
        "# Changelog\n\n## [0.7.1] - 2026-04-06\n",
    );
    write(
        &temp.path().join("crates/tq-core/Cargo.toml"),
        concat!(
            "[package]\n",
            "name = \"tq-core\"\n",
            "version.workspace = true\n",
        ),
    );

    let error = tq_release::sync_workspace_dependency_versions(temp.path())
        .expect_err("missing internal dependency entry should fail");

    assert!(
        error
            .to_string()
            .contains("workspace.dependencies is missing internal crate entries")
    );
}
