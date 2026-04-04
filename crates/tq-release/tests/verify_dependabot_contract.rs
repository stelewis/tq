use std::fs;
use std::path::Path;

fn write(path: &Path, contents: &str) {
    fs::create_dir_all(path.parent().expect("parent path must exist"))
        .expect("create parent directories");
    fs::write(path, contents).expect("write file");
}

#[test]
fn verify_dependabot_passes_for_repo_style_coverage() {
    let temp = tempfile::tempdir().expect("tempdir");

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
            "    commit-message:\n",
            "      prefix: \"chore\"\n",
        ),
    );
    write(
        &temp.path().join(".github/actions/setup-rust/action.yml"),
        "name: Setup Rust\n",
    );
    write(&temp.path().join(".github/workflows/ci.yml"), "name: CI\n");

    tq_release::verify_dependabot(temp.path()).expect("dependabot coverage should pass");
}

#[test]
fn verify_dependabot_reports_missing_required_patterns_and_uncovered_actions() {
    let temp = tempfile::tempdir().expect("tempdir");

    write(
        &temp.path().join(".github/dependabot.yml"),
        concat!(
            "version: 2\n",
            "updates:\n",
            "  - package-ecosystem: \"github-actions\"\n",
            "    directory: \"/.github/actions/setup-rust\"\n",
            "    schedule:\n",
            "      interval: \"weekly\"\n",
        ),
    );
    write(
        &temp.path().join(".github/actions/setup-rust/action.yml"),
        "name: Setup Rust\n",
    );
    write(
        &temp
            .path()
            .join(".github/actions/setup-python-uv/action.yaml"),
        "name: Setup Python\n",
    );
    write(&temp.path().join(".github/workflows/ci.yml"), "name: CI\n");

    let error =
        tq_release::verify_dependabot(temp.path()).expect_err("dependabot coverage should fail");
    let message = error.to_string();

    assert!(message.contains("does not cover local action directories"));
    assert!(message.contains("/.github/actions/setup-python-uv"));
    assert!(message.contains("must include directory pattern \"/\""));
    assert!(message.contains("must include directory pattern \"/.github/actions/*\""));
}

#[test]
fn verify_dependabot_requires_one_github_actions_update_block() {
    let temp = tempfile::tempdir().expect("tempdir");

    write(
        &temp.path().join(".github/dependabot.yml"),
        concat!(
            "version: 2\n",
            "updates:\n",
            "  - package-ecosystem: \"github-actions\"\n",
            "    directory: \"/\"\n",
            "    schedule:\n",
            "      interval: \"weekly\"\n",
            "  - package-ecosystem: \"github-actions\"\n",
            "    directory: \"/.github/actions/*\"\n",
            "    schedule:\n",
            "      interval: \"weekly\"\n",
        ),
    );

    let error = tq_release::verify_dependabot(temp.path())
        .expect_err("multiple github-actions blocks should fail");
    assert!(
        error
            .to_string()
            .contains("expected exactly one github-actions update block")
    );
}

#[test]
fn verify_dependabot_rejects_invalid_config_shape() {
    let temp = tempfile::tempdir().expect("tempdir");

    write(
        &temp.path().join(".github/dependabot.yml"),
        concat!("version: 2\n", "updates: []\n",),
    );

    let error = tq_release::verify_dependabot(temp.path()).expect_err("invalid config should fail");
    assert!(error.to_string().contains("invalid Dependabot config"));
}

#[test]
fn verify_dependabot_rejects_missing_version() {
    let temp = tempfile::tempdir().expect("tempdir");

    write(
        &temp.path().join(".github/dependabot.yml"),
        concat!(
            "updates:\n",
            "  - package-ecosystem: \"github-actions\"\n",
            "    directories:\n",
            "      - \"/\"\n",
            "      - \"/.github/actions/*\"\n",
        ),
    );

    let error =
        tq_release::verify_dependabot(temp.path()).expect_err("missing version should fail");
    let message = error.to_string();
    assert!(message.contains("invalid Dependabot config"));
    assert!(message.contains("missing Dependabot version"));
}

#[test]
fn verify_dependabot_rejects_unsupported_version() {
    let temp = tempfile::tempdir().expect("tempdir");

    write(
        &temp.path().join(".github/dependabot.yml"),
        concat!(
            "version: 3\n",
            "updates:\n",
            "  - package-ecosystem: \"github-actions\"\n",
            "    directories:\n",
            "      - \"/\"\n",
            "      - \"/.github/actions/*\"\n",
        ),
    );

    let error =
        tq_release::verify_dependabot(temp.path()).expect_err("unsupported version should fail");
    let message = error.to_string();
    assert!(message.contains("invalid Dependabot config"));
    assert!(message.contains("unsupported Dependabot version: 3"));
}

#[test]
fn verify_dependabot_rejects_inline_updates_declaration() {
    let temp = tempfile::tempdir().expect("tempdir");

    write(
        &temp.path().join(".github/dependabot.yml"),
        concat!("version: 2\n", "updates: []\n",),
    );

    let error = tq_release::verify_dependabot(temp.path())
        .expect_err("inline updates declaration should fail");
    let message = error.to_string();
    assert!(message.contains("invalid Dependabot config"));
    assert!(message.contains("updates must be declared as a block"));
}

#[test]
fn verify_dependabot_rejects_inline_directories_declaration() {
    let temp = tempfile::tempdir().expect("tempdir");

    write(
        &temp.path().join(".github/dependabot.yml"),
        concat!(
            "version: 2\n",
            "updates:\n",
            "  - package-ecosystem: \"github-actions\"\n",
            "    directories: [\"/\", \"/.github/actions/*\"]\n",
        ),
    );

    let error = tq_release::verify_dependabot(temp.path())
        .expect_err("inline directories declaration should fail");
    let message = error.to_string();
    assert!(message.contains("invalid Dependabot config"));
    assert!(message.contains("directories must be declared as a block list"));
}

#[test]
fn verify_dependabot_rejects_unknown_top_level_key() {
    let temp = tempfile::tempdir().expect("tempdir");

    write(
        &temp.path().join(".github/dependabot.yml"),
        concat!(
            "version: 2\n",
            "updates:\n",
            "  - package-ecosystem: \"github-actions\"\n",
            "    directories:\n",
            "      - \"/\"\n",
            "      - \"/.github/actions/*\"\n",
            "unexpected: true\n",
        ),
    );

    let error =
        tq_release::verify_dependabot(temp.path()).expect_err("unknown top-level keys should fail");
    let message = error.to_string();
    assert!(message.contains("invalid Dependabot config"));
    assert!(message.contains("unknown top-level key unexpected"));
}

#[test]
fn verify_dependabot_rejects_unknown_update_key() {
    let temp = tempfile::tempdir().expect("tempdir");

    write(
        &temp.path().join(".github/dependabot.yml"),
        concat!(
            "version: 2\n",
            "updates:\n",
            "  - package-ecosystem: \"github-actions\"\n",
            "    directories:\n",
            "      - \"/\"\n",
            "      - \"/.github/actions/*\"\n",
            "    unsupported-key: true\n",
        ),
    );

    let error =
        tq_release::verify_dependabot(temp.path()).expect_err("unknown update keys should fail");
    let message = error.to_string();
    assert!(message.contains("invalid Dependabot config"));
    assert!(message.contains("unknown update key unsupported-key"));
}

#[test]
fn verify_dependabot_rejects_non_list_directory_entries() {
    let temp = tempfile::tempdir().expect("tempdir");

    write(
        &temp.path().join(".github/dependabot.yml"),
        concat!(
            "version: 2\n",
            "updates:\n",
            "  - package-ecosystem: \"github-actions\"\n",
            "    directories:\n",
            "      path: \"/\"\n",
        ),
    );

    let error =
        tq_release::verify_dependabot(temp.path()).expect_err("directories must be a list block");
    let message = error.to_string();
    assert!(message.contains("invalid Dependabot config"));
    assert!(message.contains("directories entries must be list items at indent 6"));
}

#[test]
fn verify_dependabot_rejects_update_without_package_ecosystem() {
    let temp = tempfile::tempdir().expect("tempdir");

    write(
        &temp.path().join(".github/dependabot.yml"),
        concat!("version: 2\n", "updates:\n", "  - directory: \"/\"\n",),
    );

    let error = tq_release::verify_dependabot(temp.path())
        .expect_err("missing package ecosystem should fail");
    let message = error.to_string();
    assert!(message.contains("invalid Dependabot config"));
    assert!(message.contains("dependabot update is missing package-ecosystem"));
}

#[test]
fn verify_dependabot_rejects_update_with_both_directory_fields() {
    let temp = tempfile::tempdir().expect("tempdir");

    write(
        &temp.path().join(".github/dependabot.yml"),
        concat!(
            "version: 2\n",
            "updates:\n",
            "  - package-ecosystem: \"github-actions\"\n",
            "    directory: \"/\"\n",
            "    directories:\n",
            "      - \"/.github/actions/*\"\n",
        ),
    );

    let error =
        tq_release::verify_dependabot(temp.path()).expect_err("both directory shapes should fail");
    let message = error.to_string();
    assert!(message.contains("invalid Dependabot config"));
    assert!(message.contains("exactly one of directory or directories"));
}

#[test]
fn verify_dependabot_rejects_update_without_directory_fields() {
    let temp = tempfile::tempdir().expect("tempdir");

    write(
        &temp.path().join(".github/dependabot.yml"),
        concat!(
            "version: 2\n",
            "updates:\n",
            "  - package-ecosystem: \"github-actions\"\n",
            "    schedule:\n",
            "      interval: \"weekly\"\n",
        ),
    );

    let error = tq_release::verify_dependabot(temp.path())
        .expect_err("missing directory shapes should fail");
    let message = error.to_string();
    assert!(message.contains("invalid Dependabot config"));
    assert!(message.contains("exactly one of directory or directories"));
}

#[test]
fn verify_dependabot_rejects_empty_directories_list() {
    let temp = tempfile::tempdir().expect("tempdir");

    write(
        &temp.path().join(".github/dependabot.yml"),
        concat!(
            "version: 2\n",
            "updates:\n",
            "  - package-ecosystem: \"github-actions\"\n",
            "    directories:\n",
        ),
    );

    let error =
        tq_release::verify_dependabot(temp.path()).expect_err("empty directories list should fail");
    let message = error.to_string();
    assert!(message.contains("invalid Dependabot config"));
    assert!(message.contains("directories must contain at least one entry"));
}

#[test]
fn verify_dependabot_accepts_single_quoted_scalars() {
    let temp = tempfile::tempdir().expect("tempdir");

    write(
        &temp.path().join(".github/dependabot.yml"),
        concat!(
            "version: '2'\n",
            "updates:\n",
            "  - package-ecosystem: 'github-actions'\n",
            "    directories:\n",
            "      - '/'\n",
            "      - '/.github/actions/*'\n",
            "    schedule:\n",
            "      interval: 'weekly'\n",
        ),
    );
    write(
        &temp.path().join(".github/actions/setup-rust/action.yml"),
        "name: Setup Rust\n",
    );
    write(&temp.path().join(".github/workflows/ci.yml"), "name: CI\n");

    tq_release::verify_dependabot(temp.path()).expect("single-quoted scalars should be accepted");
}
