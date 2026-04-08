use std::path::{Path, PathBuf};

use tq_core::TargetName;
use tq_engine::{EngineResult, Finding, RuleId, Severity};
use tq_reporting::{JsonReporter, TextReporter, TextStyling};

fn build_finding(
    rule_id: &str,
    severity: Severity,
    message: &str,
    path: impl Into<PathBuf>,
    line: Option<u32>,
    suggestion: Option<&str>,
    target: Option<&str>,
) -> Finding {
    Finding::new(
        RuleId::parse(rule_id).expect("valid rule id"),
        severity,
        message,
        path,
        line,
        suggestion.map(str::to_owned),
        target.map(|value| TargetName::parse(value).expect("target should parse")),
    )
    .expect("valid finding")
}

#[test]
fn text_reporter_renders_findings_summary_and_optional_suggestions() {
    let result = EngineResult::new(vec![
        build_finding(
            "mapping-missing-test",
            Severity::Error,
            "Missing test file",
            PathBuf::from("/workspace/tests/tq/test_alpha.py"),
            Some(7),
            Some("Create test file at: tests/tq/test_alpha.py"),
            Some("core"),
        ),
        build_finding(
            "orphaned-test",
            Severity::Warning,
            "Test does not map to source module",
            PathBuf::from("relative/tests/test_beta.py"),
            None,
            None,
            None,
        ),
    ]);

    let mut output = Vec::new();
    TextReporter::new(Path::new("/workspace"))
        .with_suggestions(true)
        .write(&mut output, &result)
        .expect("text report should render");

    let rendered = String::from_utf8(output).expect("utf8 output");
    assert_eq!(
        rendered,
        concat!(
            "target=core tests/tq/test_alpha.py:7: error (mapping-missing-test) Missing test file ",
            "(suggestion: Create test file at: tests/tq/test_alpha.py)\n",
            "relative/tests/test_beta.py: warning (orphaned-test) Test does not map to source module\n",
            "Summary: 1 error(s), 1 warning(s), 0 info finding(s)\n",
        )
    );
}

#[test]
fn text_reporter_renders_success_message_without_summary() {
    let result = EngineResult::new(Vec::new());
    let mut output = Vec::new();

    TextReporter::new(Path::new("/workspace"))
        .write(&mut output, &result)
        .expect("text report should render");

    assert_eq!(
        String::from_utf8(output).expect("utf8 output"),
        "All checks passed!\n"
    );
}

#[test]
fn text_reporter_renders_ansi_success_message_when_enabled() {
    let result = EngineResult::new(Vec::new());
    let mut output = Vec::new();

    TextReporter::new(Path::new("/workspace"))
        .with_styling(TextStyling::Ansi)
        .write(&mut output, &result)
        .expect("text report should render");

    assert_eq!(
        String::from_utf8(output).expect("utf8 output"),
        "\u{1b}[1;36mAll checks passed!\u{1b}[0m\n"
    );
}

#[test]
fn text_reporter_renders_ansi_finding_frames_when_enabled() {
    let result = EngineResult::new(vec![build_finding(
        "mapping-missing-test",
        Severity::Error,
        "Missing test file",
        PathBuf::from("/workspace/tests/tq/test_alpha.py"),
        Some(7),
        Some("Create test file at: tests/tq/test_alpha.py"),
        Some("core"),
    )]);
    let mut output = Vec::new();

    TextReporter::new(Path::new("/workspace"))
        .with_styling(TextStyling::Ansi)
        .with_suggestions(true)
        .write(&mut output, &result)
        .expect("text report should render");

    let rendered = String::from_utf8(output).expect("utf8 output");
    assert!(rendered.contains("\u{1b}[2mtarget=core \u{1b}[0mtests/tq/test_alpha.py:7:"));
    assert!(rendered.contains("\u{1b}[31merror\u{1b}[0m (mapping-missing-test) Missing test file"));
    assert!(
        rendered.contains(
            "(\u{1b}[36msuggestion\u{1b}[0m: Create test file at: tests/tq/test_alpha.py)"
        )
    );
    assert!(rendered.contains("\u{1b}[1mSummary:\u{1b}[0m \u{1b}[31m1 error(s)\u{1b}[0m, \u{1b}[33m0 warning(s)\u{1b}[0m, \u{1b}[34m0 info finding(s)\u{1b}[0m"));
}

#[test]
fn json_reporter_preserves_field_order_and_includes_nullables() {
    let result = EngineResult::new(vec![build_finding(
        "structure-mismatch",
        Severity::Warning,
        "Move test under target root",
        PathBuf::from("/workspace/tests/pkg/test_example.py"),
        Some(3),
        None,
        Some("pkg"),
    )]);

    let mut output = Vec::new();
    JsonReporter::new(Path::new("/workspace"))
        .write(&mut output, &result)
        .expect("json report should render");

    assert_eq!(
        String::from_utf8(output).expect("utf8 output"),
        concat!(
            "{\"findings\":[{\"rule_id\":\"structure-mismatch\",\"severity\":\"warning\",",
            "\"message\":\"Move test under target root\",\"path\":\"tests/pkg/test_example.py\",",
            "\"line\":3,\"suggestion\":null,\"target\":\"pkg\"}],",
            "\"summary\":{\"errors\":0,\"warnings\":1,\"infos\":0,\"total\":1}}\n",
        )
    );
}
