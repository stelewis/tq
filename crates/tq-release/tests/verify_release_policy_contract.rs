use std::fs;
use std::path::Path;

fn write(path: &Path, contents: &str) {
    fs::create_dir_all(path.parent().expect("parent path must exist"))
        .expect("create parent directories");
    fs::write(path, contents).expect("write file");
}

#[test]
fn verify_release_policy_passes_when_workspace_and_dependabot_policies_pass() {
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
        &temp.path().join(".github/dependabot.yml"),
        concat!(
            "version: 2\n",
            "updates:\n",
            "  - package-ecosystem: \"github-actions\"\n",
            "    directories:\n",
            "      - \"/\"\n",
            "      - \"/.github/actions/*\"\n",
            "    schedule:\n",
            "      interval: \"weekly\"\n",
        ),
    );
    write(
        &temp.path().join(".github/actions/setup-rust/action.yml"),
        "name: Setup Rust\n",
    );
    write(&temp.path().join(".github/workflows/ci.yml"), "name: CI\n");

    tq_release::verify_release_policy(temp.path()).expect("release policy should pass");
}

#[test]
fn verify_release_policy_fails_when_either_policy_fails() {
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

    let error = tq_release::verify_release_policy(temp.path())
        .expect_err("release policy should fail when workspace version policy fails");

    assert!(
        error
            .to_string()
            .contains("workspace.dependencies.tq-core.version")
    );
}
