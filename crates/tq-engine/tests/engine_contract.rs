use std::path::{Path, PathBuf};

use tempfile::tempdir;
use tq_core::{RelativePathBuf, TargetName};
use tq_discovery::AnalysisIndex;
use tq_engine::{
    AnalysisContext, EngineError, EngineResult, Finding, Rule, RuleEngine, RuleId, Severity,
    TargetPlanInput, aggregate_results, plan_target_runs,
};

struct NoFindingRule {
    rule_id: RuleId,
}

impl NoFindingRule {
    fn new() -> Self {
        Self {
            rule_id: RuleId::parse("no-findings").expect("valid rule id"),
        }
    }
}

impl Rule for NoFindingRule {
    fn rule_id(&self) -> &RuleId {
        &self.rule_id
    }

    fn evaluate(&self, _context: &AnalysisContext) -> Vec<Finding> {
        Vec::new()
    }
}

struct MixedRuleA {
    rule_id: RuleId,
}

impl MixedRuleA {
    fn new() -> Self {
        Self {
            rule_id: RuleId::parse("rule-a").expect("valid rule id"),
        }
    }
}

impl Rule for MixedRuleA {
    fn rule_id(&self) -> &RuleId {
        &self.rule_id
    }

    fn evaluate(&self, _context: &AnalysisContext) -> Vec<Finding> {
        vec![
            Finding::new(
                self.rule_id.clone(),
                Severity::Warning,
                "warn later",
                PathBuf::from("tests/tq/test_beta.py"),
                Some(12),
                None,
                None,
            )
            .expect("valid finding"),
            Finding::new(
                self.rule_id.clone(),
                Severity::Error,
                "error first",
                PathBuf::from("tests/tq/test_alpha.py"),
                Some(4),
                None,
                None,
            )
            .expect("valid finding"),
        ]
    }
}

struct MixedRuleB {
    rule_id: RuleId,
}

impl MixedRuleB {
    fn new() -> Self {
        Self {
            rule_id: RuleId::parse("rule-b").expect("valid rule id"),
        }
    }
}

struct DuplicateRuleA {
    rule_id: RuleId,
}

impl DuplicateRuleA {
    fn new() -> Self {
        Self {
            rule_id: RuleId::parse("duplicate-rule").expect("valid rule id"),
        }
    }
}

impl Rule for DuplicateRuleA {
    fn rule_id(&self) -> &RuleId {
        &self.rule_id
    }

    fn evaluate(&self, _context: &AnalysisContext) -> Vec<Finding> {
        Vec::new()
    }
}

struct DuplicateRuleB {
    rule_id: RuleId,
}

impl DuplicateRuleB {
    fn new() -> Self {
        Self {
            rule_id: RuleId::parse("duplicate-rule").expect("valid rule id"),
        }
    }
}

impl Rule for DuplicateRuleB {
    fn rule_id(&self) -> &RuleId {
        &self.rule_id
    }

    fn evaluate(&self, _context: &AnalysisContext) -> Vec<Finding> {
        Vec::new()
    }
}

impl Rule for MixedRuleB {
    fn rule_id(&self) -> &RuleId {
        &self.rule_id
    }

    fn evaluate(&self, _context: &AnalysisContext) -> Vec<Finding> {
        vec![
            Finding::new(
                self.rule_id.clone(),
                Severity::Info,
                "info same path",
                PathBuf::from("tests/tq/test_alpha.py"),
                Some(2),
                None,
                None,
            )
            .expect("valid finding"),
        ]
    }
}

fn write(path: &Path) {
    std::fs::create_dir_all(path.parent().expect("parent path must exist"))
        .expect("create parent directories");
    std::fs::write(path, "pass\n").expect("write fixture file");
}

fn test_context() -> AnalysisContext {
    let temp = tempdir().expect("tempdir");
    let source_root = temp.path().join("src").join("tq");
    let test_root = temp.path().join("tests");
    std::fs::create_dir_all(&source_root).expect("create source root");
    std::fs::create_dir_all(&test_root).expect("create test root");

    let index = AnalysisIndex::create(
        &source_root,
        &test_root,
        vec![PathBuf::from("foo.py")],
        vec![PathBuf::from("tq/test_foo.py")],
    )
    .expect("index should be created");

    AnalysisContext::new(index)
}

#[test]
fn engine_no_rules_returns_empty_result() {
    let context = test_context();
    let engine = RuleEngine::new(Vec::new()).expect("engine should allow empty rule list");

    let result = engine.run(&context);

    assert!(result.findings().is_empty());
    assert_eq!(result.summary().errors(), 0);
    assert_eq!(result.summary().warnings(), 0);
    assert_eq!(result.summary().infos(), 0);
    assert_eq!(result.summary().total(), 0);
    assert!(!result.has_errors());
}

#[test]
fn engine_aggregates_and_sorts_findings_deterministically() {
    let context = test_context();
    let engine = RuleEngine::new(vec![
        Box::new(MixedRuleA::new()),
        Box::new(MixedRuleB::new()),
    ])
    .expect("engine should accept unique rule ids");

    let result = engine.run(&context);

    let found_paths = result
        .findings()
        .iter()
        .map(|finding| finding.path().to_string_lossy().to_string())
        .collect::<Vec<_>>();

    assert_eq!(
        found_paths,
        vec![
            "tests/tq/test_alpha.py".to_owned(),
            "tests/tq/test_alpha.py".to_owned(),
            "tests/tq/test_beta.py".to_owned(),
        ]
    );
    assert_eq!(result.findings()[0].line(), Some(2));
    assert_eq!(result.findings()[1].line(), Some(4));
    assert_eq!(result.findings()[2].line(), Some(12));
    assert_eq!(result.summary().errors(), 1);
    assert_eq!(result.summary().warnings(), 1);
    assert_eq!(result.summary().infos(), 1);
    assert_eq!(result.summary().total(), 3);
    assert!(result.has_errors());
}

#[test]
fn engine_executes_rule_instances() {
    let context = test_context();
    let engine = RuleEngine::new(vec![
        Box::new(NoFindingRule::new()),
        Box::new(MixedRuleB::new()),
    ])
    .expect("engine should accept unique rule ids");

    let result = engine.run(&context);

    assert_eq!(result.findings().len(), 1);
    assert_eq!(
        result.findings()[0].rule_id(),
        &RuleId::parse("rule-b").expect("valid rule id")
    );
}

#[test]
fn aggregate_results_merges_and_sorts_findings() {
    let context = test_context();
    let result_a = RuleEngine::new(vec![Box::new(MixedRuleA::new())])
        .expect("engine should accept unique rule ids")
        .run(&context);
    let result_b = RuleEngine::new(vec![Box::new(MixedRuleB::new())])
        .expect("engine should accept unique rule ids")
        .run(&context);

    let merged: EngineResult = aggregate_results(&[result_b, result_a]);

    let found_paths = merged
        .findings()
        .iter()
        .map(|finding| finding.path().to_string_lossy().to_string())
        .collect::<Vec<_>>();

    assert_eq!(
        found_paths,
        vec![
            "tests/tq/test_alpha.py".to_owned(),
            "tests/tq/test_alpha.py".to_owned(),
            "tests/tq/test_beta.py".to_owned(),
        ]
    );
    assert_eq!(merged.summary().errors(), 1);
    assert_eq!(merged.summary().warnings(), 1);
    assert_eq!(merged.summary().infos(), 1);
}

#[test]
fn engine_rejects_duplicate_rule_ids_at_construction() {
    let result = RuleEngine::new(vec![
        Box::new(DuplicateRuleA::new()),
        Box::new(DuplicateRuleB::new()),
    ]);

    let Err(error) = result else {
        panic!("engine should reject duplicate rule ids");
    };

    assert_eq!(error.to_string(), "Rule engine received duplicate rule ids");
}

#[test]
fn finding_rejects_empty_message_with_typed_error() {
    let error = Finding::new(
        RuleId::parse("rule-a").expect("valid rule id"),
        Severity::Error,
        "   ",
        PathBuf::from("tests/tq/test_alpha.py"),
        None,
        None,
        None,
    )
    .expect_err("finding should reject empty messages");

    assert!(matches!(error, EngineError::EmptyFindingMessage));
}

#[test]
fn finding_rejects_non_positive_line_with_typed_error() {
    let error = Finding::new(
        RuleId::parse("rule-a").expect("valid rule id"),
        Severity::Error,
        "valid message",
        PathBuf::from("tests/tq/test_alpha.py"),
        Some(0),
        None,
        None,
    )
    .expect_err("finding should reject line zero");

    assert!(matches!(error, EngineError::InvalidFindingLine));
}

#[test]
fn plan_target_runs_creates_context_per_active_target() {
    let temp = tempdir().expect("tempdir");

    write(&temp.path().join("src").join("tq").join("module.py"));
    write(&temp.path().join("tests").join("tq").join("test_module.py"));

    let target = TargetPlanInput::new(
        TargetName::parse("tq").expect("target name should parse"),
        RelativePathBuf::new("tq").expect("package path should parse"),
        temp.path().join("src").join("tq"),
        temp.path().join("tests"),
        PathBuf::from("tests"),
    );

    let planned = plan_target_runs(std::slice::from_ref(&target), std::slice::from_ref(&target))
        .expect("planning should succeed");

    assert_eq!(planned.len(), 1);
    let target_context = planned[0]
        .context()
        .target()
        .expect("planner must attach target context");
    assert_eq!(target_context.name().as_str(), "tq");
    assert_eq!(target_context.package_path().as_path(), Path::new("tq"));
    assert_eq!(target_context.test_root_display(), Path::new("tests"));
    assert_eq!(
        target_context.known_target_package_paths(),
        &[RelativePathBuf::new("tq").expect("package path should parse")]
    );
}

#[test]
fn plan_target_runs_preserves_nested_test_root_display() {
    let temp = tempdir().expect("tempdir");

    write(&temp.path().join("src").join("tq").join("module.py"));
    write(
        &temp
            .path()
            .join("python")
            .join("tests")
            .join("tq")
            .join("test_module.py"),
    );

    let target = TargetPlanInput::new(
        TargetName::parse("tq").expect("target name should parse"),
        RelativePathBuf::new("tq").expect("package path should parse"),
        temp.path().join("src").join("tq"),
        temp.path().join("python").join("tests"),
        PathBuf::from("python/tests"),
    );

    let planned = plan_target_runs(std::slice::from_ref(&target), std::slice::from_ref(&target))
        .expect("planning should succeed");

    assert_eq!(
        planned[0]
            .context()
            .target()
            .expect("planner must attach target context")
            .test_root_display(),
        Path::new("python/tests")
    );
}

#[test]
fn plan_target_runs_uses_configured_targets_for_known_paths() {
    let temp = tempdir().expect("tempdir");

    write(&temp.path().join("src").join("tq").join("module.py"));
    write(&temp.path().join("tests").join("tq").join("test_module.py"));
    write(&temp.path().join("scripts").join("docs").join("generate.py"));
    write(
        &temp
            .path()
            .join("tests")
            .join("scripts")
            .join("docs")
            .join("test_generate.py"),
    );

    let tq_target = TargetPlanInput::new(
        TargetName::parse("tq").expect("target name should parse"),
        RelativePathBuf::new("tq").expect("package path should parse"),
        temp.path().join("src").join("tq"),
        temp.path().join("tests"),
        PathBuf::from("tests"),
    );
    let scripts_target = TargetPlanInput::new(
        TargetName::parse("scripts").expect("target name should parse"),
        RelativePathBuf::new("scripts").expect("package path should parse"),
        temp.path().join("scripts"),
        temp.path().join("tests"),
        PathBuf::from("tests"),
    );

    let planned = plan_target_runs(
        &[tq_target, scripts_target.clone()],
        std::slice::from_ref(&scripts_target),
    )
    .expect("planning should succeed");

    assert_eq!(planned.len(), 1);
    let target_context = planned[0]
        .context()
        .target()
        .expect("planner must attach target context");
    assert_eq!(target_context.name().as_str(), "scripts");
    assert_eq!(
        target_context.known_target_package_paths(),
        &[
            RelativePathBuf::new("tq").expect("package path should parse"),
            RelativePathBuf::new("scripts").expect("package path should parse"),
        ]
    );
}
