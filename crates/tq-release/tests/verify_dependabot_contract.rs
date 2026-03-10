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
