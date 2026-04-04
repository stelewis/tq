mod support;

use std::path::PathBuf;

use tq_core::{RelativePathBuf, TargetName};
use tq_discovery::AnalysisIndex;
use tq_engine::Rule;
use tq_engine::{AnalysisContext, TargetContext};
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

    let rule = StructureMismatchRule::new().expect("rule should be valid");
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

    let rule = StructureMismatchRule::new().expect("rule should be valid");
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

    let rule = StructureMismatchRule::new().expect("rule should be valid");
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

    let rule = StructureMismatchRule::new().expect("rule should be valid");
    let findings = rule.evaluate(&context);

    assert!(findings.is_empty());
}

#[test]
fn structure_rule_preserves_nested_test_root_in_suggestion() {
    let temp = fixture_workspace();
    let source_root = temp.path().join("src").join("tq");
    let test_root = temp.path().join("python").join("tests");
    std::fs::create_dir_all(&source_root).expect("create source root");
    std::fs::create_dir_all(&test_root).expect("create test root");

    let index = AnalysisIndex::create(
        &source_root,
        &test_root,
        vec![PathBuf::from("engine/runner.py")],
        vec![PathBuf::from("tq/test_runner.py")],
    )
    .expect("index should be created");
    let context = AnalysisContext::with_target(
        index,
        TargetContext::new(
            TargetName::parse("active").expect("target name should parse"),
            RelativePathBuf::new("tq").expect("package path should parse"),
            vec![RelativePathBuf::new("tq").expect("package path should parse")],
            PathBuf::from("python/tests"),
        ),
    );

    let rule = StructureMismatchRule::new().expect("rule should be valid");
    let findings = rule.evaluate(&context);

    assert_eq!(findings.len(), 1);
    assert_eq!(
        findings[0].suggestion(),
        Some("Move to: python/tests/tq/engine/test_runner.py")
    );
}
