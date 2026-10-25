use tq_config::{CliOverrides, resolve_tq_config};
use tq_core::accepted::AcceptedFindingKind;

const TARGET: &str = r#"
[[tool.tq.targets]]
name = "app"
package = "pkg"
source_root = "src"
test_root = "tests"
"#;

fn resolve(content: &str) -> Result<tq_config::TqConfig, tq_config::ConfigError> {
    let temp = tempfile::tempdir().expect("tempdir");
    let path = temp.path().join("pyproject.toml");
    std::fs::write(&path, content).expect("write configuration");
    resolve_tq_config(
        temp.path(),
        Some(&path),
        true,
        None,
        &CliOverrides::default(),
    )
}

#[test]
fn acceptances_are_typed_and_target_local() {
    let config = resolve(&format!(
        r#"[tool.tq]
{TARGET}
accepted_findings = [
  {{ rule = "mapping-missing-test", path = "boundary.py", rationale = " Shared contract. " }},
  {{ rule = "test-file-too-large", path = "pkg/test_boundary.py", rationale = "Cohesive suite.", max_non_blank_lines = 740 }},
]
[[tool.tq.targets]]
name = "other"
package = "other"
source_root = "src"
test_root = "tests"
"#
    ))
    .expect("valid acceptances");
    let accepted = config.targets()[0].accepted_findings();
    assert_eq!(accepted.len(), 2);
    assert_eq!(accepted[0].kind(), AcceptedFindingKind::MappingMissingTest);
    assert_eq!(accepted[0].path(), std::path::Path::new("boundary.py"));
    assert_eq!(accepted[0].rationale(), "Shared contract.");
    assert!(matches!(
        accepted[1].kind(),
        AcceptedFindingKind::TestFileTooLarge { max_non_blank_lines }
            if max_non_blank_lines.get() == 740
    ));
    assert_eq!(config.targets()[1].accepted_findings(), []);
}

#[test]
fn invalid_acceptances_fail_closed() {
    let mapping = r#"rule = "mapping-missing-test", path = "boundary.py", rationale = "Reviewed.""#;
    let size =
        r#"rule = "test-file-too-large", path = "pkg/test_boundary.py", rationale = "Reviewed.""#;
    let cases = [
        ("42".to_owned(), "array of tables"),
        ("[42]".to_owned(), "must be a table"),
        (format!("[{{ {mapping}, typo = 1 }}]"), "Unknown key"),
        (format!("[{{ {mapping}, max_non_blank_lines = 4 }}]"), "Unknown key"),
        (format!("[{{ {mapping} }}, {{ {mapping} }}]"), "duplicate value"),
        (format!("[{{ {size} }}]"), "max_non_blank_lines"),
        (format!("[{{ {size}, max_non_blank_lines = 0 }}]"), "must be >= 1"),
        (format!("[{{ {size}, max_non_blank_lines = -1 }}]"), "must be >= 1"),
        (format!("[{{ {size}, max_non_blank_lines = 4.5 }}]"), "must be an integer"),
        (r#"[{rule = "orphaned-test", path = "test_a.py", rationale = "Reviewed."}]"#.to_owned(), "must be one of"),
        (r#"[{rule = "mapping-missing-test", path = "a.py"}]"#.to_owned(), "rationale"),
        (r#"[{rule = "mapping-missing-test", path = "a.py", rationale = " "}]"#.to_owned(), "non-empty"),
        (r#"[{rule = "mapping-missing-test", path = 3, rationale = "Reviewed."}]"#.to_owned(), "must be a string"),
        (r#"[{rule = "test-file-too-large", path = "module.py", rationale = "Reviewed.", max_non_blank_lines = 4}]"#.to_owned(), "test_*.py"),
    ];
    for (entries, expected) in cases {
        let error = resolve(&format!(
            "[tool.tq]\n{TARGET}\naccepted_findings = {entries}"
        ))
        .expect_err("invalid acceptance must fail");
        assert!(error.to_string().contains(expected), "{entries}: {error}");
    }
    for path in [
        "",
        "/a.py",
        "../a.py",
        "a/../b.py",
        "./a.py",
        "a/./b.py",
        "a//b.py",
        "a.py/",
        "C:/a.py",
        r"C:\a.py",
        "*.py",
        "a?.py",
        "[ab].py",
        "{a,b}.py",
        "a.txt",
    ] {
        let entry = format!(
            "{{ rule = 'mapping-missing-test', path = '{path}', rationale = 'Reviewed.' }}"
        );
        assert!(
            resolve(&format!(
                "[tool.tq]\n{TARGET}\naccepted_findings = [{entry}]"
            ))
            .is_err(),
            "must reject {path:?}"
        );
    }
    let error = resolve(&format!("[tool.tq]\naccepted_findings = []\n{TARGET}"))
        .expect_err("acceptances cannot be shared defaults");
    assert!(error.to_string().contains("Unknown [tool.tq] key"));
}
