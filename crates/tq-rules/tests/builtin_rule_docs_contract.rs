use std::collections::BTreeSet;

use tq_rules::{builtin_rule_docs, builtin_rule_ids, builtin_rule_severity_vocabulary};

#[test]
fn builtin_rule_docs_cover_the_runtime_rule_registry() {
    let docs = builtin_rule_docs();
    let doc_ids = docs.iter().map(|entry| entry.id).collect::<BTreeSet<_>>();
    let runtime_ids = builtin_rule_ids()
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

        assert_ne!(id.trim(), "");
        assert_ne!(title.trim(), "");
        assert!(allowed_severities.contains(default_severity.as_str()));
        assert_ne!(added_in.trim(), "");
        assert_ne!(behavior_changes.trim(), "");
        assert_ne!(what_it_does.trim(), "");
        assert_ne!(why_this_matters.trim(), "");
        assert_ne!(trigger_conditions.len(), 0);
        assert_ne!(examples.len(), 0);
        assert_ne!(how_to_address.len(), 0);
        assert_ne!(related_controls.len(), 0);

        for example in *examples {
            let tq_rules::RuleDocExample { source, test } = example;
            assert_ne!(source.trim(), "");
            assert_ne!(test.trim(), "");
        }

        for condition in *trigger_conditions {
            assert_ne!(condition.trim(), "");
        }

        for item in *how_to_address {
            assert_ne!(item.trim(), "");
        }

        for control in *related_controls {
            assert_ne!(control.trim(), "");
        }
    }
}
