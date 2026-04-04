mod support;

use std::collections::BTreeSet;
use std::path::PathBuf;

use tq_core::InitModulesMode;
use tq_engine::Rule;
use tq_rules::{MappingMissingTestRule, QualifierStrategy};

use crate::support::{context_with_target, create_dirs, fixture_workspace};

#[test]
fn mapping_rule_emits_error_for_unmapped_source() {
    let temp = fixture_workspace();
    let (source_root, test_root) = create_dirs(temp.path());

    let context = context_with_target(
        &source_root,
        &test_root,
        vec![PathBuf::from("alpha.py"), PathBuf::from("beta.py")],
        vec![PathBuf::from("tq/test_alpha.py")],
        "tq",
        vec!["tq".to_owned()],
    );

    let rule = MappingMissingTestRule::new(
        InitModulesMode::Ignore,
        QualifierStrategy::AnySuffix,
        BTreeSet::new(),
    )
    .expect("rule should be valid");
    let findings = rule.evaluate(&context);

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].rule_id().as_str(), "mapping-missing-test");
    assert_eq!(findings[0].severity().as_str(), "error");
    assert!(findings[0].path().ends_with("beta.py"));
}

#[test]
fn mapping_rule_allowlist_rejects_unknown_suffix_match() {
    let temp = fixture_workspace();
    let (source_root, test_root) = create_dirs(temp.path());

    let context = context_with_target(
        &source_root,
        &test_root,
        vec![PathBuf::from("engine/runner.py")],
        vec![PathBuf::from("tq/engine/test_runner_smoke.py")],
        "tq",
        vec!["tq".to_owned()],
    );

    let rule = MappingMissingTestRule::new(
        InitModulesMode::Ignore,
        QualifierStrategy::Allowlist,
        std::iter::once("regression".to_owned()).collect(),
    )
    .expect("rule should be valid");
    let findings = rule.evaluate(&context);

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].rule_id().as_str(), "mapping-missing-test");
}
