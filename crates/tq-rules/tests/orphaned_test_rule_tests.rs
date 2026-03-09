mod support;

use std::collections::BTreeSet;
use std::path::PathBuf;

use tq_engine::Rule;
use tq_rules::{OrphanedTestRule, QualifierStrategy};

use crate::support::{context_with_target, create_dirs, fixture_workspace};

#[test]
fn orphaned_rule_emits_warning_for_missing_source() {
    let temp = fixture_workspace();
    let (source_root, test_root) = create_dirs(temp.path());

    let context = context_with_target(
        &source_root,
        &test_root,
        vec![PathBuf::from("engine/runner.py")],
        vec![PathBuf::from("tq/engine/test_missing.py")],
        "tq",
        vec!["tq".to_owned()],
    );

    let rule = OrphanedTestRule::new(QualifierStrategy::None, BTreeSet::new())
        .expect("rule should be valid");
    let findings = rule.evaluate(&context);

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].rule_id().as_str(), "orphaned-test");
    assert_eq!(findings[0].severity().as_str(), "warning");
}

#[test]
fn orphaned_rule_allowlist_accepts_qualified_test() {
    let temp = fixture_workspace();
    let (source_root, test_root) = create_dirs(temp.path());

    let context = context_with_target(
        &source_root,
        &test_root,
        vec![PathBuf::from("engine/runner.py")],
        vec![PathBuf::from("tq/engine/test_runner_regression.py")],
        "tq",
        vec!["tq".to_owned()],
    );

    let rule = OrphanedTestRule::new(
        QualifierStrategy::Allowlist,
        std::iter::once("regression".to_owned()).collect(),
    )
    .expect("rule should be valid");
    let findings = rule.evaluate(&context);

    assert!(findings.is_empty());
}
