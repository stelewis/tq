mod support;

use std::num::NonZeroU64;
use std::path::PathBuf;

use tq_core::accepted::{AcceptedFinding, AcceptedFindingKind};
use tq_core::{InitModulesMode, QualifierStrategy, RuleId};
use tq_engine::{AnalysisContext, EngineResult, RuleEngine};
use tq_rules::{BuiltinRuleOptions, BuiltinRuleRegistry, RuleSelection};

use support::{context_with_test_metrics, create_dirs, fixture_workspace};

fn options() -> BuiltinRuleOptions {
    BuiltinRuleOptions::new(
        InitModulesMode::Include,
        NonZeroU64::new(3).expect("positive limit"),
        QualifierStrategy::AnySuffix,
        [],
    )
    .expect("valid options")
}

fn evaluate(
    context: &AnalysisContext,
    accepted: &[AcceptedFinding],
    selection: &RuleSelection,
) -> EngineResult {
    let rules =
        BuiltinRuleRegistry::build_rules(selection, &options(), accepted).expect("build rules");
    RuleEngine::new(rules)
        .expect("unique rules")
        .run(context)
        .expect("evaluate rules")
}

fn selection(rules: &[&'static str]) -> RuleSelection {
    RuleSelection::new(
        rules.iter().map(|id| RuleId::from_static(id)).collect(),
        vec![],
    )
}

#[test]
fn size_acceptance_is_a_ceiling_not_an_exemption() {
    let temp = fixture_workspace();
    let (source_root, test_root) = create_dirs(temp.path());
    let accepted = [AcceptedFinding::new(
        AcceptedFindingKind::TestFileTooLarge {
            max_non_blank_lines: NonZeroU64::new(5).expect("positive ceiling"),
        },
        "tq/test_boundary.py",
        "Cohesive contract.",
    )
    .expect("valid acceptance")];
    for (count, expected) in [
        (
            None,
            Some(("unused-accepted-finding", "Accepted finding is unused")),
        ),
        (
            Some(3),
            Some(("unused-accepted-finding", "Accepted finding is unused")),
        ),
        (Some(4), None),
        (Some(5), None),
        (
            Some(6),
            Some((
                "test-file-too-large",
                "Test file is too large (6 lines, accepted ceiling: 5, limit: 3)",
            )),
        ),
    ] {
        let context = context_with_test_metrics(
            &source_root,
            &test_root,
            vec![],
            count
                .into_iter()
                .map(|count| ("tq/test_boundary.py".into(), count))
                .collect(),
            "tq",
            vec![],
        );
        let result = evaluate(
            &context,
            &accepted,
            &selection(&["test-file-too-large", "unused-accepted-finding"]),
        );
        match expected {
            None => assert!(result.findings().is_empty(), "{count:?}"),
            Some((rule_id, message)) => {
                assert_eq!(result.findings().len(), 1, "{count:?}");
                assert!(
                    result.findings()[0].message().starts_with(message),
                    "{count:?}"
                );
                assert_eq!(result.findings()[0].rule_id().as_str(), rule_id);
                assert_eq!(result.summary().warnings(), 1);
            }
        }
    }
}

#[test]
fn mapping_acceptance_is_exact_and_stale_after_resolution_or_deletion() {
    let temp = fixture_workspace();
    let (source_root, test_root) = create_dirs(temp.path());
    let accepted = [AcceptedFinding::new(
        AcceptedFindingKind::MappingMissingTest,
        "boundary.py",
        "Shared contract.",
    )
    .expect("valid acceptance")];
    let cases: &[(&[&str], &[&str], usize)] = &[
        (&["boundary.py"], &[], 0),
        (&["boundary.py"], &["tq/test_boundary.py"], 1),
        (&[], &[], 1),
    ];
    for &(sources, mapped_tests, unused_count) in cases {
        let sources = sources
            .iter()
            .map(PathBuf::from)
            .chain([PathBuf::from("other/boundary.py")])
            .collect();
        let tests = mapped_tests
            .iter()
            .map(|path| (PathBuf::from(path), 0))
            .chain([(PathBuf::from("tq/test_other.py"), 4)])
            .collect();
        let context =
            context_with_test_metrics(&source_root, &test_root, sources, tests, "tq", vec![]);
        let result = evaluate(
            &context,
            &accepted,
            &selection(&[
                "mapping-missing-test",
                "test-file-too-large",
                "unused-accepted-finding",
            ]),
        );
        assert_eq!(result.findings().len(), 2 + unused_count);
        assert_eq!(
            result.summary().errors(),
            1,
            "unaccepted path still reported"
        );
        assert!(
            result
                .findings()
                .iter()
                .any(|finding| finding.path()
                    == context.index().test_root().join("tq/test_other.py"))
        );
        assert_eq!(
            result
                .findings()
                .iter()
                .filter(|finding| finding.rule_id().as_str() == "unused-accepted-finding")
                .count(),
            unused_count,
        );
    }
}

#[test]
fn inactive_rules_do_not_report_stale_acceptances() {
    let temp = fixture_workspace();
    let (source_root, test_root) = create_dirs(temp.path());
    let context = context_with_test_metrics(&source_root, &test_root, vec![], vec![], "tq", vec![]);
    let accepted = [AcceptedFinding::new(
        AcceptedFindingKind::MappingMissingTest,
        "removed.py",
        "Shared contract.",
    )
    .expect("valid acceptance")];
    for selection in [
        selection(&["test-file-too-large"]),
        selection(&["unused-accepted-finding"]),
        selection(&["mapping-missing-test"]),
        RuleSelection::new(vec![], vec![RuleId::from_static("unused-accepted-finding")]),
        RuleSelection::new(vec![], vec![RuleId::from_static("mapping-missing-test")]),
    ] {
        assert_eq!(evaluate(&context, &accepted, &selection).findings(), []);
    }
}
