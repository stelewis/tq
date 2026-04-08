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
        let tq_rules::BuiltinRuleDoc {
            id,
            title,
            default_severity,
            added_in,
            behavior_changes,
            what_it_does,
            why_this_matters,
            trigger_conditions,
            examples,
            how_to_address,
            related_controls,
        } = entry;

        assert!(!id.trim().is_empty());
        assert!(!title.trim().is_empty());
        assert!(allowed_severities.contains(default_severity.as_str()));
        assert!(!added_in.trim().is_empty());
        assert!(!behavior_changes.trim().is_empty());
        assert!(!what_it_does.trim().is_empty());
        assert!(!why_this_matters.trim().is_empty());
        assert!(!trigger_conditions.is_empty());
        assert!(!examples.is_empty());
        assert!(!how_to_address.is_empty());
        assert!(!related_controls.is_empty());

        for example in *examples {
            let tq_rules::RuleDocExample { source, test } = example;
            assert!(!source.trim().is_empty());
            assert!(!test.trim().is_empty());
        }

        for condition in *trigger_conditions {
            assert!(!condition.trim().is_empty());
        }

        for item in *how_to_address {
            assert!(!item.trim().is_empty());
        }

        for control in *related_controls {
            assert!(!control.trim().is_empty());
        }
    }
}
