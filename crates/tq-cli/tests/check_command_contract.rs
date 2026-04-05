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

fn create_multi_target_project() -> TempDir {
    let temp = TempDir::new().expect("tempdir");
    write(
        &temp.path().join("pyproject.toml"),
        r#"[tool.tq]

[[tool.tq.targets]]
name = "app"
package = "pkg"
source_root = "src"
test_root = "tests"

[[tool.tq.targets]]
name = "scripts"
package = "scripts"
source_root = "."
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

#[test]
fn check_command_respects_target_filtering() {
    let project = create_multi_target_project();
    write(
        &project.path().join("src").join("pkg").join("module.py"),
        "def run() -> None:\n    pass\n",
    );
    write(
        &project.path().join("scripts").join("generate.py"),
        "def generate() -> None:\n    pass\n",
    );

    let assert = Command::new(env!("CARGO_BIN_EXE_tq"))
        .current_dir(project.path())
        .arg("check")
        .arg("--config")
        .arg(project.path().join("pyproject.toml"))
        .arg("--output-format")
        .arg("json")
        .arg("--target")
        .arg("scripts")
        .assert();

    let output = assert.get_output();
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8(output.stdout.clone()).expect("stdout should be utf8"),
        concat!(
            "{\"findings\":[{\"rule_id\":\"mapping-missing-test\",\"severity\":\"error\",",
            "\"message\":\"No test file found for source module: generate.py\",",
            "\"path\":\"scripts/generate.py\",\"line\":null,",
            "\"suggestion\":\"Create test file at: scripts/test_generate.py\",\"target\":\"scripts\"}],",
            "\"summary\":{\"errors\":1,\"warnings\":0,\"infos\":0,\"total\":1}}\n",
        )
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn check_command_honors_init_modules_ignore_override() {
    let project = create_project();
    write(
        &project.path().join("pyproject.toml"),
        r#"[tool.tq]

[[tool.tq.targets]]
name = "app"
package = "pkg"
source_root = "src"
test_root = "tests"
init_modules = "include"
"#,
    );
    write(
        &project.path().join("src").join("pkg").join("__init__.py"),
        "def exported() -> None:\n    pass\n",
    );

    let default_assert = Command::new(env!("CARGO_BIN_EXE_tq"))
        .current_dir(project.path())
        .arg("check")
        .arg("--config")
        .arg(project.path().join("pyproject.toml"))
        .assert();
    let override_assert = Command::new(env!("CARGO_BIN_EXE_tq"))
        .current_dir(project.path())
        .arg("check")
        .arg("--config")
        .arg(project.path().join("pyproject.toml"))
        .arg("--init-modules")
        .arg("ignore")
        .assert();

    assert_eq!(default_assert.get_output().status.code(), Some(1));
    assert!(override_assert.get_output().status.success());
}

#[test]
fn check_command_honors_init_modules_include_override() {
    let project = create_project();
    write(
        &project.path().join("pyproject.toml"),
        r#"[tool.tq]

[[tool.tq.targets]]
name = "app"
package = "pkg"
source_root = "src"
test_root = "tests"
init_modules = "ignore"
"#,
    );
    write(
        &project.path().join("src").join("pkg").join("__init__.py"),
        "def exported() -> None:\n    pass\n",
    );

    let default_assert = Command::new(env!("CARGO_BIN_EXE_tq"))
        .current_dir(project.path())
        .arg("check")
        .arg("--config")
        .arg(project.path().join("pyproject.toml"))
        .assert();
    let override_assert = Command::new(env!("CARGO_BIN_EXE_tq"))
        .current_dir(project.path())
        .arg("check")
        .arg("--config")
        .arg(project.path().join("pyproject.toml"))
        .arg("--init-modules")
        .arg("include")
        .assert();

    assert!(default_assert.get_output().status.success());
    assert_eq!(override_assert.get_output().status.code(), Some(1));
}

#[test]
fn top_level_help_lists_the_check_command_cleanly() {
    let assert = Command::new(env!("CARGO_BIN_EXE_tq"))
        .arg("--help")
        .assert();

    let output = assert.get_output();
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout.clone()).expect("stdout should be utf8");
    assert!(stdout.contains("Check Python test layout quality."));
    assert!(stdout.contains("check  Run test quality checks against configured targets"));
    assert!(!stdout.contains("\n  help   "));
}

#[test]
fn check_help_groups_options_and_uses_init_modules_mode() {
    let assert = Command::new(env!("CARGO_BIN_EXE_tq"))
        .arg("check")
        .arg("--help")
        .assert();

    let output = assert.get_output();
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout.clone()).expect("stdout should be utf8");
    assert!(stdout.contains("Configuration:"));
    assert!(stdout.contains("Rule configuration:"));
    assert!(stdout.contains("Rule selection:"));
    assert!(stdout.contains("Output:"));
    assert!(stdout.contains("--init-modules <MODE>"));
    assert!(stdout.contains("[possible values: include, ignore]"));
    assert!(!stdout.contains("--ignore-init-modules"));
    assert!(!stdout.contains("--no-ignore-init-modules"));
}

#[test]
fn check_command_rejects_unknown_target_name() {
    let project = create_project();

    let assert = Command::new(env!("CARGO_BIN_EXE_tq"))
        .current_dir(project.path())
        .arg("check")
        .arg("--config")
        .arg(project.path().join("pyproject.toml"))
        .arg("--target")
        .arg("missing")
        .assert();

    let output = assert.get_output();
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("Unknown target name(s): missing"));
}

#[test]
fn check_command_rejects_duplicate_cli_select_rule_ids() {
    let project = create_project();

    let assert = Command::new(env!("CARGO_BIN_EXE_tq"))
        .current_dir(project.path())
        .arg("check")
        .arg("--config")
        .arg(project.path().join("pyproject.toml"))
        .arg("--target")
        .arg("app")
        .arg("--select")
        .arg("mapping-missing-test")
        .arg("--select")
        .arg("mapping-missing-test")
        .assert();

    let output = assert.get_output();
    assert_eq!(output.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("Duplicate rule ID in CLI values: mapping-missing-test")
    );
}

#[test]
fn check_command_uses_consistent_display_frame_for_structure_suggestions() {
    let project = create_project();
    let nested_dir = project.path().join("docs").join("dev");
    fs::create_dir_all(&nested_dir).expect("create nested cwd");
    write(
        &project
            .path()
            .join("src")
            .join("pkg")
            .join("engine")
            .join("module.py"),
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
        .current_dir(&nested_dir)
        .arg("check")
        .arg("--show-suggestions")
        .assert();

    let output = assert.get_output();
    assert_eq!(output.status.code(), Some(1));

    let stdout = String::from_utf8(output.stdout.clone()).expect("stdout should be utf8");
    let expected = fs::canonicalize(project.path().join("tests"))
        .expect("canonicalize test root")
        .join("pkg")
        .join("engine")
        .join("test_module.py")
        .to_string_lossy()
        .replace('\\', "/");
    assert!(stdout.contains(&format!("suggestion: Move to: {expected}")));
}
