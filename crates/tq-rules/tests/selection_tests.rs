use std::collections::BTreeSet;

use tq_core::InitModulesMode;
use tq_engine::RuleId;
use tq_rules::{
    BuiltinRuleOptions, BuiltinRuleRegistry, QualifierStrategy, RuleSelection, builtin_rule_ids,
    resolve_active_rule_ids,
};

#[test]
fn resolve_active_rule_ids_returns_all_builtins_when_select_is_empty() {
    let selected =
        resolve_active_rule_ids(&RuleSelection::default()).expect("selection should resolve");

    assert_eq!(
        selected,
        builtin_rule_ids().expect("built-in ids should be valid")
    );
}

#[test]
fn resolve_active_rule_ids_rejects_unknown_ids() {
    let unknown = RuleId::parse("unknown-rule").expect("valid id format");
    let selection = RuleSelection::new(vec![unknown], Vec::new());

    let error = resolve_active_rule_ids(&selection).expect_err("unknown rule ids must fail");
    assert_eq!(
        error.to_string(),
        "Unknown built-in rule ID(s): unknown-rule"
    );
}

#[test]
fn registry_builds_selected_rules_in_builtin_order() {
    let options = BuiltinRuleOptions::new(
        InitModulesMode::Ignore,
        120,
        QualifierStrategy::Allowlist,
        ["regression".to_owned()],
    )
    .expect("options should be valid");
    let selection = RuleSelection::new(
        vec![
            RuleId::parse("orphaned-test").expect("valid rule id"),
            RuleId::parse("mapping-missing-test").expect("valid rule id"),
        ],
        Vec::new(),
    );

    let rules =
        BuiltinRuleRegistry::build_rules(&selection, &options).expect("rule build should succeed");

    let ids = rules
        .iter()
        .map(|rule| rule.rule_id().as_str().to_owned())
        .collect::<Vec<_>>();

    assert_eq!(
        ids,
        vec![
            "mapping-missing-test".to_owned(),
            "orphaned-test".to_owned()
        ]
    );
}

#[test]
fn options_validate_allowlist_requires_qualifiers() {
    let error = BuiltinRuleOptions::new(
        InitModulesMode::Include,
        600,
        QualifierStrategy::Allowlist,
        BTreeSet::new(),
    )
    .expect_err("empty allowlist should fail");

    assert_eq!(
        error.to_string(),
        "allowed_qualifiers must be non-empty for allowlist strategy"
    );
}
