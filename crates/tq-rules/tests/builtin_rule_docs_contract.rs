use std::collections::BTreeSet;

use tq_rules::{builtin_rule_docs, builtin_rule_ids, builtin_rule_severity_vocabulary};

#[test]
fn builtin_rule_docs_cover_the_runtime_rule_registry() {
    let docs = builtin_rule_docs();
    let doc_ids = docs.iter().map(|entry| entry.id).collect::<BTreeSet<_>>();
    let runtime_ids = builtin_rule_ids()
        .expect("builtin rule ids")
        .into_iter()
        .map(|rule_id| rule_id.to_string())
        .collect::<BTreeSet<_>>();

    assert_eq!(docs.len(), runtime_ids.len());
    assert_eq!(
        doc_ids,
        runtime_ids
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>()
    );
}

#[test]
fn builtin_rule_docs_define_complete_non_empty_contract_text() {
    let allowed_severities = builtin_rule_severity_vocabulary()
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();

    for entry in builtin_rule_docs() {
        assert!(!entry.id.trim().is_empty());
        assert!(!entry.title.trim().is_empty());
        assert!(allowed_severities.contains(entry.default_severity));
        assert!(!entry.added_in.trim().is_empty());
        assert!(!entry.behavior_changes.trim().is_empty());
        assert!(!entry.what_it_does.trim().is_empty());
        assert!(!entry.why_this_matters.trim().is_empty());
        assert!(!entry.trigger_conditions.is_empty());
        assert!(!entry.examples.is_empty());
        assert!(!entry.how_to_address.is_empty());
        assert!(!entry.related_controls.is_empty());
    }
}
