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
            "version = \"0.7.0\"\n",
            "rust-version = \"1.96\"\n",
            "\n",
            "[workspace.dependencies]\n",
            "tq-core = { version = \"0.7.0\", path = \"crates/tq-core\" }\n",
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
            "  - package-ecosystem: \"pre-commit\"\n",
            "    directory: \"/\"\n",
            "    schedule:\n",
            "      interval: \"weekly\"\n",
            "  - package-ecosystem: \"uv\"\n",
            "    directory: \"/\"\n",
            "    schedule:\n",
            "      interval: \"weekly\"\n",
            "  - package-ecosystem: \"cargo\"\n",
            "    directory: \"/\"\n",
            "    schedule:\n",
            "      interval: \"weekly\"\n",
            "  - package-ecosystem: \"rust-toolchain\"\n",
            "    directory: \"/\"\n",
            "    schedule:\n",
            "      interval: \"weekly\"\n",
            "  - package-ecosystem: \"npm\"\n",
            "    directory: \"/\"\n",
            "    schedule:\n",
            "      interval: \"weekly\"\n",
        ),
    );
    write(
        &temp.path().join(".github/dev-tools.toml"),
        concat!(
            "[schema]\n",
            "version = 1\n",
            "\n",
            "[tools]\n",
            "rust = \"1.96.1\"\n",
            "python = \"3.14.6\"\n",
            "uv = \"0.11.28\"\n",
            "node = \"26.4.0\"\n",
            "npm = \"11.17.0\"\n",
            "mise = \"2026.7.5\"\n",
            "\n",
            "[rust-maintenance]\n",
            "cargo-outdated = \"0.17.0\"\n",
            "cargo-audit-rev = \"c9f8506963a0050a7b53ce4d3781baa9485c7a89\"\n",
            "cargo-deny = \"0.19.0\"\n",
        ),
    );
    write(
        &temp.path().join("rust-toolchain.toml"),
        concat!(
            "[toolchain]\n",
            "channel = \"1.96.1\"\n",
            "components = [\"rustfmt\", \"clippy\"]\n",
        ),
    );
    write(
        &temp.path().join("mise.toml"),
        concat!(
            "[tools]\n",
            "node = \"26.4.0\"\n",
            "python = \"3.14.6\"\n",
            "uv = \"0.11.28\"\n",
        ),
    );
    write(
        &temp.path().join("package.json"),
        concat!(
            "{\n",
            "  \"packageManager\": \"npm@11.17.0\",\n",
            "  \"engines\": {\n",
            "    \"node\": \"26.4.0\",\n",
            "    \"npm\": \"11.17.0\"\n",
            "  }\n",
            "}\n",
        ),
    );
    write(
        &temp
            .path()
            .join(".github/actions/setup-python-uv/action.yml"),
        concat!(
            "inputs:\n",
            "  python-version:\n",
            "    default: \"3.14.6\"\n",
            "runs:\n",
            "  using: composite\n",
            "  steps:\n",
            "    - uses: astral-sh/setup-uv@example\n",
            "      with:\n",
            "        version: \"0.11.28\"\n",
        ),
    );
    write(
        &temp
            .path()
            .join(".github/actions/setup-rust-maintenance-tools/action.yml"),
        concat!(
            "inputs:\n",
            "  cargo-outdated-version:\n",
            "    default: \"0.17.0\"\n",
            "  cargo-audit-rev:\n",
            "    default: \"c9f8506963a0050a7b53ce4d3781baa9485c7a89\"\n",
            "  cargo-deny-version:\n",
            "    default: \"0.19.0\"\n",
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
            "version = \"0.7.0\"\n",
            "\n",
            "[workspace.dependencies]\n",
            "tq-core = { version = \"0.6.3\", path = \"crates/tq-core\" }\n",
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

    let error = tq_release::verify_release_policy(temp.path())
        .expect_err("release policy should fail when workspace version policy fails");

    assert!(
        error
            .to_string()
            .contains("workspace.dependencies.tq-core.version")
    );
}
