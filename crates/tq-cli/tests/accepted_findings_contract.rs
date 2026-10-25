use std::fs;

use assert_cmd::Command;
use serde_json::{Value, json};

fn check(project: &std::path::Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_tq"))
        .current_dir(project)
        .args(["check", "--config", "pyproject.toml"])
        .args(args)
        .output()
        .expect("run tq")
}

fn json_output(output: &std::process::Output) -> Value {
    assert!(output.stderr.is_empty(), "{output:?}");
    serde_json::from_slice(&output.stdout).expect("valid JSON output")
}

fn accepted_project() -> tempfile::TempDir {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();
    fs::create_dir_all(root.join("src/pkg")).expect("source root");
    fs::create_dir_all(root.join("tests/pkg")).expect("test root");
    fs::write(root.join("src/pkg/boundary.py"), "pass\n").expect("source module");
    fs::write(root.join("tests/pkg/test_contract.py"), "a\nb\nc\n").expect("contract suite");
    fs::write(
        root.join("pyproject.toml"),
        r#"[tool.tq]
select = ["mapping-missing-test", "test-file-too-large", "unused-accepted-finding"]
max_test_file_non_blank_lines = 2
fail_on = "info"

[[tool.tq.targets]]
name = "app"
package = "pkg"
source_root = "src"
test_root = "tests"

[[tool.tq.targets.accepted_findings]]
rule = "mapping-missing-test"
path = "boundary.py"
rationale = "Shared contract."

[[tool.tq.targets.accepted_findings]]
rule = "test-file-too-large"
path = "pkg/test_contract.py"
max_non_blank_lines = 3
rationale = "Cohesive contract."
"#,
    )
    .expect("configuration");
    temp
}

#[test]
fn accepted_findings_are_quiet_until_growth_exceeds_the_ceiling() {
    let temp = accepted_project();
    let root = temp.path();
    let clean = check(root, &["--output-format", "json"]);
    assert!(clean.status.success());
    assert_eq!(
        json_output(&clean),
        json!({
            "findings": [],
            "summary": {"errors": 0, "warnings": 0, "infos": 0, "total": 0}
        })
    );
    let text = check(root, &[]);
    assert!(text.status.success());
    assert_eq!(text.stdout, b"All checks passed!\n");

    fs::write(root.join("tests/pkg/test_contract.py"), "a\nb\nc\nd\n").expect("grow suite");
    let grown = check(root, &["--output-format", "json"]);
    assert_eq!(grown.status.code(), Some(1));
    assert_eq!(
        json_output(&grown),
        json!({
            "findings": [{
                "rule_id": "test-file-too-large", "severity": "warning",
                "message": "Test file is too large (4 lines, accepted ceiling: 3, limit: 2)",
                "path": "tests/pkg/test_contract.py", "line": null,
                "suggestion": "Split this module into smaller focused test files", "target": "app"
            }],
            "summary": {"errors": 0, "warnings": 1, "infos": 0, "total": 1}
        })
    );
}

#[test]
fn stale_acceptances_respect_severity_overrides_and_rule_selection() {
    let temp = accepted_project();
    let root = temp.path();
    fs::write(root.join("tests/pkg/test_boundary.py"), "pass\n").expect("resolve mapping");
    fs::remove_file(root.join("tests/pkg/test_contract.py")).expect("remove oversized suite");
    let stale = check(
        root,
        &[
            "--output-format",
            "json",
            "--severity",
            "mapping-missing-test=error",
        ],
    );
    assert_eq!(stale.status.code(), Some(1));
    let stale = json_output(&stale);
    assert_eq!(
        stale["summary"],
        json!({"errors": 0, "warnings": 2, "infos": 0, "total": 2})
    );
    let findings = stale["findings"].as_array().expect("findings array");
    for finding in findings {
        assert_eq!(finding["rule_id"], "unused-accepted-finding");
        assert_eq!(finding["severity"], "warning");
        assert!(
            finding["message"]
                .as_str()
                .expect("message")
                .starts_with("Accepted finding is unused:")
        );
        assert_eq!(
            finding["suggestion"],
            "Remove this entry from accepted_findings"
        );
        assert_eq!(finding["target"], "app");
    }
    let demoted = check(
        root,
        &[
            "--output-format",
            "json",
            "--severity",
            "unused-accepted-finding=info",
            "--fail-on",
            "warning",
        ],
    );
    assert!(demoted.status.success());
    assert_eq!(
        json_output(&demoted)["summary"],
        json!({"errors": 0, "warnings": 0, "infos": 2, "total": 2})
    );
    let text = check(root, &["--show-suggestions"]);
    assert_eq!(text.status.code(), Some(1));
    let text = String::from_utf8(text.stdout).expect("text output");
    assert!(text.contains("unused-accepted-finding"));
    assert!(text.contains("Remove this entry from accepted_findings"));
    let ignored = check(root, &["--ignore", "unused-accepted-finding"]);
    assert!(ignored.status.success());
    assert_eq!(ignored.stdout, b"All checks passed!\n");
}

#[test]
fn ignoring_unused_entries_preserves_acceptance_and_growth_detection() {
    let temp = accepted_project();
    let root = temp.path();
    fs::write(root.join("tests/pkg/test_boundary.py"), "pass\n").expect("resolve mapping");
    let clean = check(root, &["--ignore", "unused-accepted-finding"]);
    assert!(clean.status.success());
    assert_eq!(clean.stdout, b"All checks passed!\n");

    fs::write(root.join("tests/pkg/test_contract.py"), "a\nb\nc\nd\n").expect("grow suite");
    let grown = check(
        root,
        &[
            "--ignore",
            "unused-accepted-finding",
            "--output-format",
            "json",
        ],
    );
    assert_eq!(grown.status.code(), Some(1));
    let output = json_output(&grown);
    assert_eq!(output["findings"].as_array().expect("findings").len(), 1);
    assert_eq!(output["findings"][0]["rule_id"], "test-file-too-large");
    assert_eq!(
        output["summary"],
        json!({"errors": 0, "warnings": 1, "infos": 0, "total": 1})
    );
}

#[test]
fn acceptance_does_not_cross_targets_and_paths_do_not_depend_on_cwd() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();
    for package in ["first", "second"] {
        fs::create_dir_all(root.join("src").join(package)).expect("source package");
        fs::write(root.join("src").join(package).join("boundary.py"), "pass\n").expect("module");
    }
    fs::create_dir_all(root.join("tests/nested")).expect("test root and nested cwd");
    fs::write(root.join("pyproject.toml"), r#"[tool.tq]
select = ["mapping-missing-test"]

[[tool.tq.targets]]
name = "first"
package = "first"
source_root = "src"
test_root = "tests"
accepted_findings = [{ rule = "mapping-missing-test", path = "boundary.py", rationale = "Shared contract." }]

[[tool.tq.targets]]
name = "second"
package = "second"
source_root = "src"
test_root = "tests"
"#).expect("configuration");
    let output = Command::new(env!("CARGO_BIN_EXE_tq"))
        .current_dir(root.join("tests/nested"))
        .args([
            "check",
            "--config",
            "../../pyproject.toml",
            "--output-format",
            "json",
        ])
        .output()
        .expect("run from nested cwd");
    assert_eq!(output.status.code(), Some(1));
    let output = json_output(&output);
    assert_eq!(output["findings"].as_array().expect("findings").len(), 1);
    assert_eq!(output["findings"][0]["target"], "second");
    assert_eq!(output["findings"][0]["rule_id"], "mapping-missing-test");
    let first_only = check(root, &["--target", "first"]);
    assert!(first_only.status.success());
    assert_eq!(first_only.stdout, b"All checks passed!\n");
}
