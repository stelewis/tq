mod support;

use std::path::PathBuf;

use tq_engine::Rule;
use tq_rules::StructureMismatchRule;

use crate::support::{context_with_target, create_dirs, fixture_workspace};

#[test]
fn structure_rule_emits_warning_for_misplaced_test() {
    let temp = fixture_workspace();
    let (source_root, test_root) = create_dirs(temp.path());

    let context = context_with_target(
        &source_root,
        &test_root,
        vec![PathBuf::from("engine/runner.py")],
        vec![PathBuf::from("tq/test_runner.py")],
        "tq",
        vec!["tq".to_owned()],
    );

    let rule = StructureMismatchRule::new();
    let findings = rule.evaluate(&context);

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].rule_id().as_str(), "structure-mismatch");
    assert_eq!(findings[0].severity().as_str(), "warning");
    assert_eq!(
        findings[0].suggestion(),
        Some("Move to: tests/tq/engine/test_runner.py")
    );
}

#[test]
fn structure_rule_allows_correctly_placed_test() {
    let temp = fixture_workspace();
    let (source_root, test_root) = create_dirs(temp.path());

    let context = context_with_target(
        &source_root,
        &test_root,
        vec![PathBuf::from("engine/runner.py")],
        vec![PathBuf::from("tq/engine/test_runner.py")],
        "tq",
        vec!["tq".to_owned()],
    );

    let rule = StructureMismatchRule::new();
    let findings = rule.evaluate(&context);

    assert!(findings.is_empty());
}

#[test]
fn structure_rule_skips_non_unit_scopes() {
    let temp = fixture_workspace();
    let (source_root, test_root) = create_dirs(temp.path());

    let context = context_with_target(
        &source_root,
        &test_root,
        vec![PathBuf::from("engine/runner.py")],
        vec![
            PathBuf::from("integration/test_runner.py"),
            PathBuf::from("e2e/test_runner.py"),
        ],
        "tq",
        vec!["tq".to_owned()],
    );

    let rule = StructureMismatchRule::new();
    let findings = rule.evaluate(&context);

    assert!(findings.is_empty());
}

#[test]
fn structure_rule_ignores_sibling_target_tests() {
    let temp = fixture_workspace();
    let (source_root, test_root) = create_dirs(temp.path());

    let context = context_with_target(
        &source_root,
        &test_root,
        vec![PathBuf::from("docs/generate.py")],
        vec![
            PathBuf::from("tq/engine/test_runner.py"),
            PathBuf::from("scripts/docs/test_generate.py"),
        ],
        "scripts",
        vec!["tq".to_owned(), "scripts".to_owned()],
    );

    let rule = StructureMismatchRule::new();
    let findings = rule.evaluate(&context);

    assert!(findings.is_empty());
}
