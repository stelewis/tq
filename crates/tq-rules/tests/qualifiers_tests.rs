use std::collections::BTreeSet;

use tq_rules::{QualifierStrategy, candidate_module_names};

#[test]
fn candidate_module_names_none_strategy_returns_only_full_stem() {
    let names = candidate_module_names(
        "runner_regression",
        QualifierStrategy::None,
        &BTreeSet::new(),
    );

    assert_eq!(names, vec!["runner_regression".to_owned()]);
}

#[test]
fn candidate_module_names_any_suffix_strategy_returns_prefixes() {
    let names = candidate_module_names(
        "income_record_validation_smoke",
        QualifierStrategy::AnySuffix,
        &BTreeSet::new(),
    );

    assert_eq!(
        names,
        vec![
            "income_record_validation_smoke".to_owned(),
            "income_record_validation".to_owned(),
            "income_record".to_owned(),
            "income".to_owned(),
        ]
    );
}

#[test]
fn candidate_module_names_allowlist_strategy_filters_suffixes() {
    let allowed = std::iter::once("regression".to_owned()).collect::<BTreeSet<_>>();
    let names = candidate_module_names("runner_regression", QualifierStrategy::Allowlist, &allowed);

    assert_eq!(
        names,
        vec!["runner_regression".to_owned(), "runner".to_owned()]
    );
}
