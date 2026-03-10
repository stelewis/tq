use std::fs;
use std::path::Path;

use assert_cmd::Command;
use tempfile::TempDir;

fn write(path: &Path, contents: &str) {
    fs::create_dir_all(path.parent().expect("parent path must exist"))
        .expect("create parent directories");
    fs::write(path, contents).expect("write file");
}

fn create_project() -> TempDir {
    let temp = TempDir::new().expect("tempdir");
    write(
        &temp.path().join("pyproject.toml"),
        r#"[tool.tq]

[[tool.tq.targets]]
name = "app"
package = "pkg"
source_root = "src"
test_root = "tests"
"#,
    );
    fs::create_dir_all(temp.path().join("src").join("pkg")).expect("create source package root");
    fs::create_dir_all(temp.path().join("tests")).expect("create test root");
    temp
}

#[test]
fn check_command_reports_success_for_clean_project() {
    let project = create_project();
    write(
        &project.path().join("src").join("pkg").join("module.py"),
        "def run() -> None:\n    pass\n",
    );
    write(
        &project
            .path()
            .join("tests")
            .join("pkg")
            .join("test_module.py"),
        "def test_run() -> None:\n    assert True\n",
    );

    let assert = Command::new(env!("CARGO_BIN_EXE_tq"))
        .current_dir(project.path())
        .arg("check")
        .arg("--config")
        .arg(project.path().join("pyproject.toml"))
        .assert();

    let output = assert.get_output();
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout.clone()).expect("stdout should be utf8"),
        "All checks passed!\n"
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn check_command_emits_json_and_exit_code_one_for_findings() {
    let project = create_project();
    write(
        &project.path().join("src").join("pkg").join("module.py"),
        "def run() -> None:\n    pass\n",
    );

    let assert = Command::new(env!("CARGO_BIN_EXE_tq"))
        .current_dir(project.path())
        .arg("check")
        .arg("--config")
        .arg(project.path().join("pyproject.toml"))
        .arg("--output-format")
        .arg("json")
        .assert();

    let output = assert.get_output();
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8(output.stdout.clone()).expect("stdout should be utf8"),
        concat!(
            "{\"findings\":[{\"rule_id\":\"mapping-missing-test\",\"severity\":\"error\",",
            "\"message\":\"No test file found for source module: module.py\",",
            "\"path\":\"src/pkg/module.py\",\"line\":null,",
            "\"suggestion\":\"Create test file at: pkg/test_module.py\",\"target\":\"app\"}],",
            "\"summary\":{\"errors\":1,\"warnings\":0,\"infos\":0,\"total\":1}}\n",
        )
    );
    assert!(output.stderr.is_empty());
}
