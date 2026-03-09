mod support;

use std::path::PathBuf;

use tq_engine::Rule;
use tq_rules::TestFileTooLargeRule;

use crate::support::{context_with_target, create_dirs, fixture_workspace};

#[test]
fn file_too_large_rule_counts_non_blank_non_comment_lines() {
    let temp = fixture_workspace();
    let (source_root, test_root) = create_dirs(temp.path());
    let test_file = test_root.join("tq").join("test_big.py");
    std::fs::create_dir_all(test_root.join("tq")).expect("create test package dir");
    std::fs::write(
        test_file,
        "\n# comment\n\ndef test_one():\n    assert True\n\ndef test_two():\n    assert True\n",
    )
    .expect("write test file");

    let context = context_with_target(
        &source_root,
        &test_root,
        vec![PathBuf::from("alpha.py")],
        vec![PathBuf::from("tq/test_big.py")],
        "tq",
        vec!["tq".to_owned()],
    );

    let rule = TestFileTooLargeRule::new(3).expect("rule should be valid");
    let findings = rule.evaluate(&context);

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].rule_id().as_str(), "test-file-too-large");
    assert_eq!(findings[0].severity().as_str(), "warning");
}

#[test]
fn file_too_large_rule_emits_warning_for_unreadable_file() {
    let temp = fixture_workspace();
    let (source_root, test_root) = create_dirs(temp.path());
    std::fs::create_dir_all(test_root.join("tq")).expect("create test package dir");
    std::fs::write(
        test_root.join("tq").join("test_bad_encoding.py"),
        [0xff, 0xfe, 0xfa],
    )
    .expect("write unreadable bytes");

    let context = context_with_target(
        &source_root,
        &test_root,
        vec![PathBuf::from("alpha.py")],
        vec![PathBuf::from("tq/test_bad_encoding.py")],
        "tq",
        vec!["tq".to_owned()],
    );

    let rule = TestFileTooLargeRule::new(3).expect("rule should be valid");
    let findings = rule.evaluate(&context);

    assert_eq!(findings.len(), 1);
    assert!(
        findings[0]
            .message()
            .contains("Could not read test file for size check")
    );
}
