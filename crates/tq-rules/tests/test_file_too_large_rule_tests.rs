mod support;

use std::num::NonZeroU64;
use std::path::PathBuf;

use tq_engine::Rule;
use tq_rules::TestFileTooLargeRule;

use crate::support::{context_with_test_metrics, create_dirs, fixture_workspace};

#[test]
fn file_too_large_rule_uses_discovery_line_metrics() {
    let temp = fixture_workspace();
    let (source_root, test_root) = create_dirs(temp.path());
    let test_path = PathBuf::from("tq/test_big.py");

    let context = context_with_test_metrics(
        &source_root,
        &test_root,
        vec![PathBuf::from("alpha.py")],
        vec![(test_path, 4)],
        "tq",
        vec!["tq".to_owned()],
    );

    let rule = TestFileTooLargeRule::new(NonZeroU64::new(3).expect("non-zero literal"));
    let findings = rule
        .evaluate(&context)
        .expect("rule evaluation should succeed");

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].rule_id().as_str(), "test-file-too-large");
    assert_eq!(findings[0].severity().as_str(), "warning");
}

#[test]
fn file_too_large_rule_does_not_read_files_during_evaluation() {
    let temp = fixture_workspace();
    let (source_root, test_root) = create_dirs(temp.path());
    let relative_path = PathBuf::from("tq/test_removed.py");
    let full_path = test_root.join(&relative_path);
    std::fs::create_dir_all(full_path.parent().expect("test file parent"))
        .expect("create test package dir");
    std::fs::write(&full_path, "def test_one():\n    assert True\n").expect("write test file");

    let context = context_with_test_metrics(
        &source_root,
        &test_root,
        vec![PathBuf::from("alpha.py")],
        vec![(relative_path, 4)],
        "tq",
        vec!["tq".to_owned()],
    );
    std::fs::remove_file(full_path).expect("remove file after discovery snapshot");

    let rule = TestFileTooLargeRule::new(NonZeroU64::new(3).expect("non-zero literal"));
    let findings = rule
        .evaluate(&context)
        .expect("rule evaluation should use snapshot data");

    assert_eq!(findings.len(), 1);
    assert!(findings[0].path().ends_with("test_removed.py"));
}
